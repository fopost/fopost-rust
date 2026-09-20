//! `client.google_business()` — manage a connected Google Business Profile location.
//!
//! Google grants Business Profile API access per project. Until that grant
//! lands on a deployment every call here fails with a 503 `configuration_error`.
//!
//! Responses relay Google's own shape, field for field, so they come back as
//! [`serde_json::Value`] rather than structs we would have to keep chasing.

use reqwest::Method;
use serde_json::{json, Map, Value};

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::AccountMoved;

/// The daily metrics fetched when a caller names none.
pub const DEFAULT_DAILY_METRICS: [&str; 7] = [
    "BUSINESS_IMPRESSIONS_DESKTOP_MAPS",
    "BUSINESS_IMPRESSIONS_DESKTOP_SEARCH",
    "BUSINESS_IMPRESSIONS_MOBILE_MAPS",
    "BUSINESS_IMPRESSIONS_MOBILE_SEARCH",
    "CALL_CLICKS",
    "WEBSITE_CLICKS",
    "BUSINESS_DIRECTION_REQUESTS",
];

/// One profile patch. A field left `None` keeps its value.
#[derive(Debug, Clone, Default)]
pub struct UpdateLocation {
    /// The business name shown on the listing.
    pub title: Option<String>,
    /// The profile description. `Some(None)` clears it.
    pub description: Option<Option<String>>,
    /// The website the listing links to. `Some(None)` clears it.
    pub website_uri: Option<Option<String>>,
    /// The phone number on the listing. `Some(None)` clears it.
    pub primary_phone: Option<Option<String>>,
    /// Up to two further phone numbers.
    pub additional_phones: Option<Vec<String>>,
    /// The merchant's own code for this location. `Some(None)` clears it.
    pub store_code: Option<Option<String>>,
    /// Opening hours, as Google's own period objects.
    pub regular_hours: Option<Vec<Value>>,
}

impl UpdateLocation {
    fn body(&self) -> Value {
        let mut map = Map::new();
        if let Some(title) = &self.title {
            map.insert("title".into(), json!(title));
        }
        for (key, value) in [
            ("description", &self.description),
            ("website_uri", &self.website_uri),
            ("primary_phone", &self.primary_phone),
            ("store_code", &self.store_code),
        ] {
            if let Some(value) = value {
                map.insert(key.into(), json!(value));
            }
        }
        if let Some(phones) = &self.additional_phones {
            map.insert("additional_phones".into(), json!(phones));
        }
        if let Some(hours) = &self.regular_hours {
            map.insert("regular_hours".into(), json!(hours));
        }
        Value::Object(map)
    }
}

/// One place action link to add.
#[derive(Debug, Clone)]
pub struct CreatePlaceAction {
    /// Where the button sends the visitor.
    pub uri: String,
    /// `APPOINTMENT`, `FOOD_ORDERING`, `SHOP_ONLINE` and the rest.
    pub place_action_type: String,
    /// Whether Google should prefer this link over the others of its type.
    pub is_preferred: Option<bool>,
}

/// A verification to start.
#[derive(Debug, Clone, Default)]
pub struct StartVerification {
    /// `ADDRESS`, `EMAIL`, `PHONE_CALL`, `SMS`, `AUTO` or `VETTED_PARTNER`.
    pub method: String,
    /// The language Google should send the verification in.
    pub language_code: Option<String>,
    /// The number to call or text, for the phone methods.
    pub phone_number: Option<String>,
    /// The address to mail, for the email method.
    pub email_address: Option<String>,
    /// Who the postcard is addressed to, for the address method.
    pub mailer_contact_name: Option<String>,
}

/// Manage one connected Business Profile location.
#[derive(Debug, Clone)]
pub struct GoogleBusiness<'a> {
    pub(crate) http: &'a HttpClient,
}

impl GoogleBusiness<'_> {
    /// The connected location, in the Business Information shape.
    pub async fn get_location(&self, account_id: &str) -> Result<Value> {
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/location"),
            None,
            None,
        )
        .await
    }

    /// Patch the profile; only the fields the request carries change.
    pub async fn update_location(&self, account_id: &str, input: &UpdateLocation) -> Result<Value> {
        self.read(
            Method::PATCH,
            &format!("/accounts/{account_id}/gbp/location"),
            None,
            Some(input.body()),
        )
        .await
    }

    /// The attribute values set on the location. `available` lists what Google
    /// offers for its category and region instead.
    pub async fn get_attributes(
        &self,
        account_id: &str,
        available: bool,
        category_name: Option<&str>,
        region_code: Option<&str>,
        language_code: Option<&str>,
    ) -> Result<Value> {
        let mut query: Query = Vec::new();
        if available {
            query.push(("available", "true".into()));
        }
        push_opt(&mut query, "category_name", category_name);
        push_opt(&mut query, "region_code", region_code);
        push_opt(&mut query, "language_code", language_code);
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/attributes"),
            Some(query),
            None,
        )
        .await
    }

    /// Only the named attributes change; every other one is left alone.
    pub async fn update_attributes(&self, account_id: &str, attributes: &[Value]) -> Result<Value> {
        self.read(
            Method::PATCH,
            &format!("/accounts/{account_id}/gbp/attributes"),
            None,
            Some(json!({ "attributes": attributes })),
        )
        .await
    }

    /// The location's food menus.
    pub async fn get_menus(&self, account_id: &str) -> Result<Value> {
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/menus"),
            None,
            None,
        )
        .await
    }

    /// Google has no per-section patch, so the whole menu set is replaced.
    pub async fn replace_menus(&self, account_id: &str, menus: &[Value]) -> Result<Value> {
        self.read(
            Method::PUT,
            &format!("/accounts/{account_id}/gbp/menus"),
            None,
            Some(json!({ "menus": menus })),
        )
        .await
    }

    /// The location's service list.
    pub async fn get_services(&self, account_id: &str) -> Result<Value> {
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/services"),
            None,
            None,
        )
        .await
    }

    /// Replace the whole service list.
    pub async fn replace_services(
        &self,
        account_id: &str,
        service_items: &[Value],
    ) -> Result<Value> {
        self.read(
            Method::PUT,
            &format!("/accounts/{account_id}/gbp/services"),
            None,
            Some(json!({ "service_items": service_items })),
        )
        .await
    }

    /// The photos on the location.
    pub async fn list_media(
        &self,
        account_id: &str,
        page_size: Option<u32>,
        page_token: Option<&str>,
    ) -> Result<Value> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "page_size", page_size);
        push_opt(&mut query, "page_token", page_token);
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/media"),
            Some(query),
            None,
        )
        .await
    }

    /// Add a photo from the media library; JPEG or PNG, same workspace.
    pub async fn add_media(
        &self,
        account_id: &str,
        media_id: &str,
        category: Option<&str>,
        description: Option<&str>,
    ) -> Result<Value> {
        let mut body = Map::new();
        body.insert("media_id".into(), json!(media_id));
        body.insert("category".into(), json!(category.unwrap_or("ADDITIONAL")));
        if let Some(description) = description {
            body.insert("description".into(), json!(description));
        }
        self.read(
            Method::POST,
            &format!("/accounts/{account_id}/gbp/media"),
            None,
            Some(Value::Object(body)),
        )
        .await
    }

    /// Remove a photo by the media key Google returned.
    pub async fn delete_media(&self, account_id: &str, media_key: &str) -> Result<Value> {
        self.read(
            Method::DELETE,
            &format!("/accounts/{account_id}/gbp/media/{media_key}"),
            None,
            None,
        )
        .await
    }

    /// The Book, Order and Reserve links on the listing.
    pub async fn list_place_actions(&self, account_id: &str) -> Result<Value> {
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/place-actions"),
            None,
            None,
        )
        .await
    }

    /// Add an action link to the listing.
    pub async fn create_place_action(
        &self,
        account_id: &str,
        input: &CreatePlaceAction,
    ) -> Result<Value> {
        let mut body = Map::new();
        body.insert("uri".into(), json!(input.uri));
        body.insert("place_action_type".into(), json!(input.place_action_type));
        if let Some(preferred) = input.is_preferred {
            body.insert("is_preferred".into(), json!(preferred));
        }
        self.read(
            Method::POST,
            &format!("/accounts/{account_id}/gbp/place-actions"),
            None,
            Some(Value::Object(body)),
        )
        .await
    }

    /// Patch one action link; a `None` field is left alone.
    pub async fn update_place_action(
        &self,
        account_id: &str,
        link_id: &str,
        uri: Option<&str>,
        is_preferred: Option<bool>,
    ) -> Result<Value> {
        let mut body = Map::new();
        if let Some(uri) = uri {
            body.insert("uri".into(), json!(uri));
        }
        if let Some(preferred) = is_preferred {
            body.insert("is_preferred".into(), json!(preferred));
        }
        self.read(
            Method::PATCH,
            &format!("/accounts/{account_id}/gbp/place-actions/{link_id}"),
            None,
            Some(Value::Object(body)),
        )
        .await
    }

    /// Remove one action link.
    pub async fn delete_place_action(&self, account_id: &str, link_id: &str) -> Result<Value> {
        self.read(
            Method::DELETE,
            &format!("/accounts/{account_id}/gbp/place-actions/{link_id}"),
            None,
            None,
        )
        .await
    }

    /// The ways Google will let this location be verified.
    pub async fn get_verification_options(
        &self,
        account_id: &str,
        language_code: Option<&str>,
    ) -> Result<Value> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "language_code", language_code);
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/verification"),
            Some(query),
            None,
        )
        .await
    }

    /// Start a verification; the response names the pending one.
    pub async fn start_verification(
        &self,
        account_id: &str,
        input: &StartVerification,
    ) -> Result<Value> {
        let mut body = Map::new();
        body.insert("method".into(), json!(input.method));
        for (key, value) in [
            ("language_code", &input.language_code),
            ("phone_number", &input.phone_number),
            ("email_address", &input.email_address),
            ("mailer_contact_name", &input.mailer_contact_name),
        ] {
            if let Some(value) = value {
                body.insert(key.into(), json!(value));
            }
        }
        self.read(
            Method::POST,
            &format!("/accounts/{account_id}/gbp/verification/start"),
            None,
            Some(Value::Object(body)),
        )
        .await
    }

    /// Complete a pending verification with the PIN Google sent.
    pub async fn complete_verification(
        &self,
        account_id: &str,
        verification_name: &str,
        pin: &str,
    ) -> Result<Value> {
        self.read(
            Method::POST,
            &format!("/accounts/{account_id}/gbp/verification/complete"),
            None,
            Some(json!({ "verification_name": verification_name, "pin": pin })),
        )
        .await
    }

    /// Daily impressions, calls, direction requests and clicks for the range.
    pub async fn get_performance(
        &self,
        account_id: &str,
        start_date: &str,
        end_date: &str,
        daily_metrics: &[&str],
    ) -> Result<Value> {
        let mut query: Query = vec![
            ("start_date", start_date.to_string()),
            ("end_date", end_date.to_string()),
        ];
        for metric in daily_metrics {
            query.push(("daily_metrics", (*metric).to_string()));
        }
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/performance"),
            Some(query),
            None,
        )
        .await
    }

    /// The search terms people used to find the listing, by month.
    pub async fn get_search_keywords(
        &self,
        account_id: &str,
        start_date: &str,
        end_date: &str,
        page_token: Option<&str>,
    ) -> Result<Value> {
        let mut query: Query = vec![
            ("keywords", "true".into()),
            ("start_date", start_date.to_string()),
            ("end_date", end_date.to_string()),
        ];
        push_opt(&mut query, "page_token", page_token);
        self.read(
            Method::GET,
            &format!("/accounts/{account_id}/gbp/performance"),
            Some(query),
            None,
        )
        .await
    }

    /// Hand the location to another workspace the caller owns. The connection
    /// and every row keyed to it move in one transaction.
    pub async fn assign(&self, account_id: &str, workspace_id: &str) -> Result<AccountMoved> {
        let payload = json!({ "workspace_id": workspace_id });
        let body: Envelope<AccountMoved> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{account_id}/gbp/assign"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    async fn read(
        &self,
        method: Method,
        path: &str,
        query: Option<Query>,
        body: Option<Value>,
    ) -> Result<Value> {
        let envelope: Envelope<Value> = match &body {
            Some(body) => self.http.send(method, path, query, Some(body)).await?,
            None => self.http.send::<_, ()>(method, path, query, None).await?,
        };
        Ok(envelope.data)
    }
}
