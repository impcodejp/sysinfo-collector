use crate::model::{AdapterInfo, NetworkInfo};
use anyhow::Result;
use std::net::Ipv4Addr;
use windows::Win32::NetworkManagement::IpHelper::{
    GetAdaptersAddresses, GAA_FLAG_INCLUDE_GATEWAYS, IP_ADAPTER_ADDRESSES_LH,
};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR_IN, SOCKADDR_IN6};

pub fn collect_network() -> Result<NetworkInfo> {
    let hostname = get_hostname();
    let adapters = get_adapter_infos()?;
    Ok(NetworkInfo { hostname, adapters })
}

fn get_hostname() -> String {
    use windows::Win32::System::SystemInformation::{ComputerNameDnsHostname, GetComputerNameExW};

    let mut size: u32 = 256;
    let mut buf = vec![0u16; size as usize];
    unsafe {
        if GetComputerNameExW(
            ComputerNameDnsHostname,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        )
        .is_ok()
        {
            String::from_utf16_lossy(&buf[..size as usize])
        } else {
            "UNKNOWN".to_string()
        }
    }
}

fn get_adapter_infos() -> Result<Vec<AdapterInfo>> {
    // まずバッファサイズを取得
    let mut buf_len: u32 = 0;
    unsafe {
        let _ = GetAdaptersAddresses(
            AF_UNSPEC.0 as u32,
            GAA_FLAG_INCLUDE_GATEWAYS,
            None,
            None,
            &mut buf_len,
        );
    }

    if buf_len == 0 {
        return Ok(vec![]);
    }

    let mut buf: Vec<u8> = vec![0u8; buf_len as usize];
    let result = unsafe {
        GetAdaptersAddresses(
            AF_UNSPEC.0 as u32,
            GAA_FLAG_INCLUDE_GATEWAYS,
            None,
            Some(buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
            &mut buf_len,
        )
    };

    if result != 0 {
        anyhow::bail!("GetAdaptersAddresses failed: {}", result);
    }

    let mut adapters = Vec::new();
    let mut current = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;

    while !current.is_null() {
        let adapter = unsafe { &*current };

        // ループバック (24) とトンネル (131) は除外
        if adapter.IfType == 24 || adapter.IfType == 131 {
            current = adapter.Next;
            continue;
        }

        let name = unsafe { adapter.FriendlyName.to_string().unwrap_or_default() };

        let mac_address = if adapter.PhysicalAddressLength > 0 {
            let mac_bytes = &adapter.PhysicalAddress[..adapter.PhysicalAddressLength as usize];
            Some(
                mac_bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(":"),
            )
        } else {
            None
        };

        let mut ipv4: Option<String> = None;
        let mut ipv6: Option<String> = None;
        let mut subnet_mask: Option<String> = None;

        let mut unicast = adapter.FirstUnicastAddress;
        while !unicast.is_null() {
            let ua = unsafe { &*unicast };
            let sockaddr = ua.Address.lpSockaddr;
            if !sockaddr.is_null() {
                let sa = unsafe { &*sockaddr };
                match sa.sa_family {
                    AF_INET => {
                        if ipv4.is_none() {
                            let sa_in = sockaddr as *const SOCKADDR_IN;
                            let addr = unsafe { (*sa_in).sin_addr.S_un.S_addr };
                            let ip = Ipv4Addr::from(u32::from_be(addr));
                            ipv4 = Some(ip.to_string());

                            let prefix_len = ua.OnLinkPrefixLength;
                            if prefix_len <= 32 {
                                let mask = if prefix_len == 0 {
                                    0u32
                                } else {
                                    !0u32 << (32 - prefix_len)
                                };
                                subnet_mask = Some(Ipv4Addr::from(mask).to_string());
                            }
                        }
                    }
                    AF_INET6 => {
                        if ipv6.is_none() {
                            let sa_in6 = sockaddr as *const SOCKADDR_IN6;
                            let bytes = unsafe { (*sa_in6).sin6_addr.u.Byte };
                            let ip = std::net::Ipv6Addr::from(bytes);
                            ipv6 = Some(ip.to_string());
                        }
                    }
                    _ => {}
                }
            }
            unicast = ua.Next;
        }

        // デフォルトゲートウェイ
        let mut gateway: Option<String> = None;
        let mut gw_ptr = adapter.FirstGatewayAddress;
        while !gw_ptr.is_null() {
            let gw = unsafe { &*gw_ptr };
            let sockaddr = gw.Address.lpSockaddr;
            if !sockaddr.is_null() {
                let sa = unsafe { &*sockaddr };
                if sa.sa_family == AF_INET {
                    let sa_in = sockaddr as *const SOCKADDR_IN;
                    let addr = unsafe { (*sa_in).sin_addr.S_un.S_addr };
                    let ip = Ipv4Addr::from(u32::from_be(addr));
                    gateway = Some(ip.to_string());
                    break;
                }
            }
            gw_ptr = gw.Next;
        }

        adapters.push(AdapterInfo {
            name,
            ipv4,
            ipv6,
            subnet_mask,
            gateway,
            mac_address,
            dns_primary: None,
            dns_secondary: None,
        });

        current = adapter.Next;
    }

    enrich_dns_info(&mut adapters)?;

    Ok(adapters)
}

fn enrich_dns_info(adapters: &mut Vec<AdapterInfo>) -> Result<()> {
    use serde::Deserialize;
    use wmi::{COMLibrary, WMIConnection};

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_NetworkAdapterConfiguration")]
    #[allow(non_snake_case)]
    struct NetworkAdapterConfig {
        Description: Option<String>,
        DNSServerSearchOrder: Option<Vec<String>>,
    }

    let com = COMLibrary::new()?;
    let wmi = WMIConnection::new(com)?;

    let results: Vec<NetworkAdapterConfig> = wmi.query()?;

    for config in results {
        let desc = match &config.Description {
            Some(d) => d.clone(),
            None => continue,
        };
        let dns = match &config.DNSServerSearchOrder {
            Some(d) if !d.is_empty() => d.clone(),
            _ => continue,
        };

        for adapter in adapters.iter_mut() {
            if adapter.dns_primary.is_none()
                && (adapter.name.contains(&desc) || desc.contains(&adapter.name))
            {
                adapter.dns_primary = dns.get(0).cloned();
                adapter.dns_secondary = dns.get(1).cloned();
                break;
            }
        }
    }

    Ok(())
}
