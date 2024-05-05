use std::sync::Arc;

use miette::*;
use tokio::{net::tcp::WriteHalf, sync::Mutex};

use super::{FTPCommand, InnerConnection, StatusCode};

pub struct User<'a>(&'a str);

impl<'a> FTPCommand<'a> for User<'a> {
    const KEYWORD: &'static str = "USER";

    async fn run<'b>(
        &self,
        _connection: Arc<Mutex<InnerConnection>>,
        _writer: &mut WriteHalf<'b>,
    ) -> Result<Option<StatusCode>> {
        Ok(Some(StatusCode::UserLoggedIn))
    }
}

impl<'a> TryFrom<(&'a str, Vec<&'a str>)> for User<'a> {
    type Error = miette::Error;

    fn try_from((command, args): (&'a str, Vec<&'a str>)) -> Result<Self> {
        if command == Self::KEYWORD {
            if args.len() == 1 {
                Ok(Self(args[0]))
            } else {
                Err(miette!("Invalid number of arguments"))
            }
        } else {
            Err(miette!("Invalid command"))
        }
    }
}