use argh::FromArgs;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "jwt")]
/// JWT (JSON Web Token) utilities
pub struct InspectForJwt {
  #[argh(subcommand)]
  pub subcommand: JwtCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum JwtCommand {
  Decode(DecodeArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "decode")]
/// Decode a JWT and display the header and payload
pub struct DecodeArgs {
  #[argh(positional)]
  /// the JWT to decode, or a file path containing the JWT
  input: String,
}

pub fn handle_jwt_command(command: JwtCommand) -> Result<(), String> {
  match command {
    JwtCommand::Decode(args) => decode_jwt(args),
  }
}

fn decode_jwt(args: DecodeArgs) -> Result<(), String> {
  let token = if Path::new(&args.input).is_file() {
    match fs::read_to_string(&args.input) {
      Ok(contents) => contents.trim().to_string(),
      Err(e) => {
        return Err(format!("Failed to read file '{}': {}", &args.input, e));
      }
    }
  } else {
    args.input
  };

  let header = match decode_header(&token) {
    Ok(h) => h,
    Err(e) => {
      return Err(format!("Failed to decode token header: {e}"));
    }
  };

  let mut validation = Validation::default();
  validation.insecure_disable_signature_validation();
  validation.validate_exp = false;
  validation.validate_aud = false;

  let token_data = match decode::<Value>(&token, &DecodingKey::from_secret(&[]), &validation) {
    Ok(t) => t,
    Err(e) => {
      return Err(format!("Failed to decode token: {e}"));
    }
  };

  println!("Header: {}", serde_json::to_string_pretty(&header).unwrap());
  println!("Payload: {}", serde_json::to_string_pretty(&token_data.claims).unwrap());
  Ok(())
}
