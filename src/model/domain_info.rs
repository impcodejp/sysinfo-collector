/// ドメインにおける当該マシンの役割。
/// Win32_ComputerSystem.DomainRole プロパティの値（0–5）に対応する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainRole {
    /// ワークグループ参加のワークステーション（DomainRole = 0）
    StandaloneWorkstation,
    /// ドメイン参加のワークステーション（DomainRole = 1）
    MemberWorkstation,
    /// ワークグループ参加のサーバー（DomainRole = 2）
    StandaloneServer,
    /// ドメイン参加のメンバーサーバー（DomainRole = 3）
    MemberServer,
    /// バックアップ（セカンダリ）ドメインコントローラー（DomainRole = 4）
    BackupDomainController,
    /// プライマリドメインコントローラー（PDC エミュレーター含む）（DomainRole = 5）
    PrimaryDomainController,
}

impl DomainRole {
    /// Win32_ComputerSystem.DomainRole の数値から変換する。
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::StandaloneWorkstation),
            1 => Some(Self::MemberWorkstation),
            2 => Some(Self::StandaloneServer),
            3 => Some(Self::MemberServer),
            4 => Some(Self::BackupDomainController),
            5 => Some(Self::PrimaryDomainController),
            _ => None,
        }
    }

    /// ドメイン参加かどうかを返す。
    pub fn is_domain_joined(&self) -> bool {
        matches!(
            self,
            Self::MemberWorkstation
                | Self::MemberServer
                | Self::BackupDomainController
                | Self::PrimaryDomainController
        )
    }

    /// ドメインコントローラーかどうかを返す。
    #[allow(dead_code)]
    pub fn is_domain_controller(&self) -> bool {
        matches!(
            self,
            Self::BackupDomainController | Self::PrimaryDomainController
        )
    }

    /// 表示用の日本語ラベルを返す。
    pub fn label(&self) -> &'static str {
        match self {
            Self::StandaloneWorkstation => "ワークグループ（ワークステーション）",
            Self::MemberWorkstation => "ドメインメンバー（ワークステーション）",
            Self::StandaloneServer => "ワークグループ（サーバー）",
            Self::MemberServer => "ドメインメンバー（サーバー）",
            Self::BackupDomainController => "ドメインコントローラー（セカンダリ / BDC）",
            Self::PrimaryDomainController => "ドメインコントローラー（プライマリ / PDC）",
        }
    }
}

/// ワークグループ／ドメイン参加情報。
#[derive(Debug, Clone)]
pub struct DomainInfo {
    /// ワークグループ名またはドメイン名。
    pub name: String,
    /// 当該マシンのドメインロール。
    pub role: DomainRole,
}

impl DomainInfo {
    /// ドメイン参加かどうかを返す（`role` の委譲）。
    pub fn is_domain_joined(&self) -> bool {
        self.role.is_domain_joined()
    }

    /// ドメインコントローラーかどうかを返す（`role` の委譲）。
    #[allow(dead_code)]
    pub fn is_domain_controller(&self) -> bool {
        self.role.is_domain_controller()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_role_from_u32() {
        assert_eq!(DomainRole::from_u32(0), Some(DomainRole::StandaloneWorkstation));
        assert_eq!(DomainRole::from_u32(1), Some(DomainRole::MemberWorkstation));
        assert_eq!(DomainRole::from_u32(2), Some(DomainRole::StandaloneServer));
        assert_eq!(DomainRole::from_u32(3), Some(DomainRole::MemberServer));
        assert_eq!(DomainRole::from_u32(4), Some(DomainRole::BackupDomainController));
        assert_eq!(DomainRole::from_u32(5), Some(DomainRole::PrimaryDomainController));
        assert_eq!(DomainRole::from_u32(6), None);
    }

    #[test]
    fn test_is_domain_joined() {
        assert!(!DomainRole::StandaloneWorkstation.is_domain_joined());
        assert!(!DomainRole::StandaloneServer.is_domain_joined());
        assert!(DomainRole::MemberWorkstation.is_domain_joined());
        assert!(DomainRole::MemberServer.is_domain_joined());
        assert!(DomainRole::BackupDomainController.is_domain_joined());
        assert!(DomainRole::PrimaryDomainController.is_domain_joined());
    }

    #[test]
    fn test_is_domain_controller() {
        assert!(!DomainRole::StandaloneWorkstation.is_domain_controller());
        assert!(!DomainRole::MemberServer.is_domain_controller());
        assert!(DomainRole::BackupDomainController.is_domain_controller());
        assert!(DomainRole::PrimaryDomainController.is_domain_controller());
    }

    #[test]
    fn test_domain_info_delegation() {
        let member = DomainInfo {
            name: "example.local".to_string(),
            role: DomainRole::MemberServer,
        };
        assert!(member.is_domain_joined());
        assert!(!member.is_domain_controller());

        let pdc = DomainInfo {
            name: "example.local".to_string(),
            role: DomainRole::PrimaryDomainController,
        };
        assert!(pdc.is_domain_joined());
        assert!(pdc.is_domain_controller());
    }

    #[test]
    fn test_label_returns_japanese() {
        assert!(DomainRole::StandaloneWorkstation.label().contains("ワークグループ"));
        assert!(DomainRole::PrimaryDomainController.label().contains("プライマリ"));
    }
}
