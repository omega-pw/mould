pub mod invite_user;
pub mod query_user;
pub mod read_user;

pub mod enums {
    use serde::{Deserialize, Serialize};
    use std::fmt;

    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum UserSource {
        System = 1,   //系统用户
        External = 2, //外部用户
    }
    impl fmt::Display for UserSource {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    UserSource::System => "系统用户",
                    UserSource::External => "外部用户",
                }
            )
        }
    }

    #[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
    pub enum ProviderType {
        Openid = 1, //Open Id
        Oauth2 = 2, //Oauth2
    }
    impl fmt::Display for ProviderType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    ProviderType::Openid => "Open Id",
                    ProviderType::Oauth2 => "Oauth2",
                }
            )
        }
    }
}
