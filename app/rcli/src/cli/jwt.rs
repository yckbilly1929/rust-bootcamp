use crate::{get_reader, process_jwt_sign, process_jwt_verify, CmdExector};

use super::verify_file;
use clap::Parser;
use enum_dispatch::enum_dispatch;
use jsonwebtoken::Algorithm;
use std::time::Duration;

// #[derive(Debug, Clone, Copy)]
// pub enum JwtAlgorithm {
//     Alg(Algorithm),
// }

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExector)]
pub enum JwtSubCommand {
    #[command(about = "Sign a text with a private/session key and return a signature")]
    Sign(JwtSignOpts),
    #[command(about = "Verify a signature with a public/session key")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(short, long, default_value = "EdDSA")]
    pub alg: Algorithm,
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub key: String,
    // claim
    #[arg(long)]
    pub sub: String,
    #[arg(long)]
    pub aud: String,
    #[arg(long, value_parser = humantime::parse_duration)]
    pub exp: Duration,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long, default_value = "EdDSA")]
    pub alg: Algorithm,
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub key: String,
    #[arg(short, long)]
    pub token: String,
    // optional
    #[arg(long)]
    pub sub: String,
    #[arg(long)]
    pub aud: String,
}

// fn parse_jwt_alg(format: &str) -> Result<JwtAlgorithm, anyhow::Error> {
//     format.parse()
// }

// impl From<Algorithm> for JwtAlgorithm {
//     fn from(algorithm: Algorithm) -> Self {
//         JwtAlgorithm::Alg(algorithm)
//     }
// }

// impl From<JwtAlgorithm> for Algorithm {
//     fn from(algorithm: JwtAlgorithm) -> Self {
//         match algorithm {
//             JwtAlgorithm::Alg(alg) => alg,
//             _ => panic!("Cannot convert non-Algorithm variant to Algorithm"),
//         }
//     }
// }

// impl FromStr for JwtAlgorithm {
//     type Err = anyhow::Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "HS256" => Ok(JwtAlgorithm::Alg(Algorithm::HS256)),
//             "EdDSA" => Ok(JwtAlgorithm::Alg(Algorithm::EdDSA)),
//             _ => Err(anyhow::anyhow!("Invalid format")),
//         }
//     }
// }

// impl From<JwtAlgorithm> for &'static str {
//     fn from(format: JwtAlgorithm) -> Self {
//         match format {
//             JwtAlgorithm::Alg(Algorithm::HS256) => "HS256",
//             JwtAlgorithm::Alg(Algorithm::EdDSA) => "EdDSA",
//             _ => "EdDSA", // TODO: can we make this panic?
//         }
//     }
// }

// impl fmt::Display for JwtAlgorithm {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", Into::<&str>::into(*self))
//     }
// }

impl CmdExector for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mut reader = get_reader(&self.key)?;
        let token = process_jwt_sign(&mut reader, self.alg, &self.sub, &self.aud, self.exp)?;
        println!("{}", token);
        Ok(())
    }
}

impl CmdExector for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let mut reader = get_reader(&self.key)?;
        process_jwt_verify(&mut reader, self.alg, &self.token, &self.sub, &self.aud)?;
        println!("✓ JWT verified");
        Ok(())
    }
}
