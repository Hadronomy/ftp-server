use miette::*;

use tracing::*;

use crate::types::{System, SystemType};
use crate::{FTPCommand, InnerConnectionRef, StatusCode};

pub struct Syst;

impl<'a> FTPCommand<'a> for Syst {
    const KEYWORD: &'static str = "SYST";

    async fn run<'b>(
        &self,
        _connection: InnerConnectionRef,
        _writer: &mut tokio::net::tcp::WriteHalf<'b>,
    ) -> Result<Option<StatusCode>> {
        trace!(
            "Reporting {} system type",
            System::from(SystemType::from_os())
        );
        Ok(Some(StatusCode::SystemType(SystemType::from_os())))
    }
}

impl<'a> TryFrom<(&'a str, Vec<&'a str>)> for Syst {
    type Error = miette::Error;

    fn try_from((command, args): (&'a str, Vec<&'a str>)) -> Result<Self> {
        if command == Self::KEYWORD {
            if args.is_empty() {
                Ok(Self)
            } else {
                Err(miette!("Invalid number of arguments"))
            }
        } else {
            Err(miette!("Invalid command"))
        }
    }
}
