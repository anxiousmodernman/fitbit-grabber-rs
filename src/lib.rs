extern crate chrono;
extern crate reqwest;
extern crate oauth2;
extern crate url;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate tiny_http;


use std::error::Error;

use reqwest::header::{Authorization, Bearer, Headers, UserAgent};
use reqwest::Method;
use chrono::{NaiveDate, Utc};
use oauth2::{AuthType, Config};

#[derive(Serialize, Deserialize, Debug)]
pub struct Token(oauth2::Token);

pub struct FitbitClient {
    client: reqwest::Client,
    base: url::Url,
}

impl FitbitClient {
    pub fn new(token: Token) -> FitbitClient {
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer {
            token: token.0.access_token,
        }));
        headers.set(UserAgent::new("fitbit-grabber-rs (0.1.0)"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Unable to build HTTP client");

        FitbitClient {
            client: client,
            base: url::Url::parse("https://api.fitbit.com/1/").unwrap(),
        }
    }

    pub fn user(&self) -> Result<String, String> {
        let url = self.base.join("user/-/profile.json").map_err(stringify)?;
        self.client
            .request(reqwest::Method::Get, url)
            .send()
            .and_then(|mut r| r.text())
            .map_err(stringify)
    }

    pub fn heart(&self, date: NaiveDate) -> Result<String, String> {
        let path = format!(
            "user/-/activities/heart/date/{}/1d.json",
            date.format("%Y-%m-%d")
        );
        let url = self.base.join(&path).map_err(stringify)?;
        self.client
            .request(Method::Get, url)
            .send()
            .and_then(|mut r| r.text())
            .map_err(stringify)
    }

    pub fn step(&self, date: NaiveDate) -> Result<String, String> {
        let path = format!(
            "user/-/activities/steps/date/{}/1d.json",
            date.format("%Y-%m-%d")
        );
        let url = self.base.join(&path).map_err(stringify)?;
        self.client
            .request(Method::Get, url)
            .send()
            .and_then(|mut r| r.text())
            .map_err(stringify)
    }

    pub fn weight(&self, date:NaiveDate) -> Result<String, String> {
        let path = format!(
            "user/-/body/weight/date/{}/1d.json",
            date.format("%Y-%m-%d")
        );
        unimplemented!()
    }
}

pub struct FitbitAuth(oauth2::Config);

impl FitbitAuth {
    pub fn new(client_id: &str, client_secret: &str) -> FitbitAuth {
        let auth_url = "https://www.fitbit.com/oauth2/authorize";
        let token_url = "https://api.fitbit.com/oauth2/token";
        // let token_url = "http://localhost:8080";

        // Set up the config for the Github OAuth2 process.
        let mut config = Config::new(client_id, client_secret, auth_url, token_url);

        // config = config.set_response_type(ResponseType::Token);
        config = config.set_auth_type(AuthType::BasicAuth);

        // This example is requesting access to the user's public repos and email.
        config = config.add_scope("activity");
        config = config.add_scope("heartrate");
        config = config.add_scope("profile");

        // This example will be running its own server at localhost:8080.
        // See below for the server implementation.
        config = config.set_redirect_url("http://localhost:8080");

        FitbitAuth(config)
    }

    pub fn get_token(&self) -> Result<oauth2::Token, String> {
        let authorize_url = self.0.authorize_url();

        println!(
            "Open this URL in your browser:\n{}\n",
            authorize_url.to_string()
        );

        // FIXME avoid unwrap here
        let server = tiny_http::Server::http("localhost:8080").unwrap();
        let request = server.recv().map_err(stringify)?;
        let url = request.url().to_string();
        let response = tiny_http::Response::from_string("Go back to your terminal :)");
        request.respond(response).map_err(stringify)?;

        let code = {
            // remove leading '/?'
            let mut parsed = url::form_urlencoded::parse(url[2..].as_bytes());

            let (_, value) = parsed
                .find(|pair| {
                    let &(ref key, _) = pair;
                    key == "code"
                })
                .ok_or("query param `code` not found")?;
            value.to_string()
        };

        // Exchange the code with a token.
        self.0.exchange_code(code).map_err(stringify)
    }

    pub fn exchange_refresh_token(&self, token: Token) -> Result<oauth2::Token, String> {
        match token.0.refresh_token {
            Some(t) => self.0.exchange_refresh_token(t).map_err(stringify),
            None => Err("No refresh token available".to_string()),
        }
    }
}

fn stringify<E: Error>(e: E) -> String {
    format!("{}", e)
}


#[cfg(test)]
mod tests {
    use DateRange;

    use chrono::{NaiveDate, Utc};

    #[test]
    fn daterange() {
        let d = DateRange::new("2017-09-01", "2017-09-30").unwrap();
        assert_eq!(d.start, NaiveDate::from_ymd(2017, 9, 1));
        assert_eq!(d.end, NaiveDate::from_ymd(2017, 9, 30));
    }

    #[test]
    fn daterange_from() {
        let d = DateRange::from(NaiveDate::from_ymd(2017, 9, 1));
        assert_eq!(d.start, NaiveDate::from_ymd(2017, 9, 1));
        assert_eq!(d.end, Utc::today().naive_utc());
    }

    #[test]
    fn daterange_iter() {
        let d = DateRange::new("2017-09-01", "2017-09-03").unwrap();
        let dates: Vec<NaiveDate> = d.collect();
        assert_eq!(
            dates,
            vec![
                NaiveDate::from_ymd(2017, 9, 1),
                NaiveDate::from_ymd(2017, 9, 2),
                NaiveDate::from_ymd(2017, 9, 3),
            ]
        )
    }
}