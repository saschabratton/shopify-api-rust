//! Customer resource implementation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::CustomerAddress;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CustomerState {
    Disabled,
    Invited,
    #[default]
    Enabled,
    Declined,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct EmailMarketingConsent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SmsMarketingConsent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_collected_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Customer {
    #[serde(skip_serializing)]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing)]
    pub state: Option<CustomerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_email: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exempt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exemptions: Option<Vec<String>>,
    #[serde(skip_serializing)]
    pub orders_count: Option<u64>,
    #[serde(skip_serializing)]
    pub total_spent: Option<String>,
    #[serde(skip_serializing)]
    pub last_order_id: Option<u64>,
    #[serde(skip_serializing)]
    pub last_order_name: Option<String>,
    #[serde(skip_serializing)]
    pub currency: Option<String>,
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<CustomerAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_address: Option<CustomerAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_marketing_consent: Option<EmailMarketingConsent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms_marketing_consent: Option<SmsMarketingConsent>,
}

impl RestResource for Customer {
    type Id = u64;
    type FindParams = CustomerFindParams;
    type AllParams = CustomerListParams;
    type CountParams = CustomerCountParams;

    const NAME: &'static str = "Customer";
    const PLURAL: &'static str = "customers";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "customers/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "customers"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "customers/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "customers",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "customers/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "customers/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CustomerFindParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CustomerListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CustomerCountParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_customer_struct_serialization() {
        let customer = Customer {
            id: Some(12345),
            email: Some("customer@example.com".into()),
            first_name: Some("John".into()),
            last_name: Some("Doe".into()),
            phone: Some("+1-555-555-5555".into()),
            state: Some(CustomerState::Enabled),
            tags: Some("vip, loyal".into()),
            note: Some("Great customer".into()),
            verified_email: Some(true),
            tax_exempt: Some(false),
            tax_exemptions: Some(vec!["CA_STATE_EXEMPTION".into()]),
            orders_count: Some(10),
            total_spent: Some("1234.56".into()),
            last_order_id: Some(99999),
            last_order_name: Some("#1001".into()),
            currency: Some("USD".into()),
            created_at: None,
            updated_at: None,
            admin_graphql_api_id: Some("gid://shopify/Customer/12345".into()),
            addresses: None,
            default_address: None,
            email_marketing_consent: None,
            sms_marketing_consent: None,
        };

        let json = serde_json::to_string(&customer).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["email"], "customer@example.com");
        assert_eq!(parsed["first_name"], "John");
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("state").is_none());
    }

    #[test]
    fn test_customer_deserialization_with_nested_addresses() {
        let json_str = concat!(
            r#"{"id":207119551,"email":"bob@example.com","first_name":"Bob","#,
            r#""last_name":"Norman","phone":"+16135551212","state":"enabled","#,
            r#""tags":"important","note":"VIP","verified_email":true,"tax_exempt":false,"#,
            r#""orders_count":1,"total_spent":"199.65","currency":"USD","#,
            r#""addresses":[{"id":111,"customer_id":207119551,"first_name":"Bob","#,
            r#""last_name":"Norman","city":"Louisville","default":true}]}"#
        );

        let customer: Customer = serde_json::from_str(json_str).unwrap();

        assert_eq!(customer.id, Some(207119551));
        assert_eq!(customer.email.as_deref(), Some("bob@example.com"));
        assert_eq!(customer.first_name.as_deref(), Some("Bob"));
        assert_eq!(customer.state, Some(CustomerState::Enabled));
        assert_eq!(customer.note.as_deref(), Some("VIP"));

        let addresses = customer.addresses.unwrap();
        assert_eq!(addresses.len(), 1);
        assert_eq!(addresses[0].id, Some(111));
        assert_eq!(addresses[0].default, Some(true));
    }

    #[test]
    fn test_customer_state_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&CustomerState::Disabled).unwrap(),
            "\"disabled\""
        );
        assert_eq!(
            serde_json::to_string(&CustomerState::Invited).unwrap(),
            "\"invited\""
        );
        assert_eq!(
            serde_json::to_string(&CustomerState::Enabled).unwrap(),
            "\"enabled\""
        );
        assert_eq!(
            serde_json::to_string(&CustomerState::Declined).unwrap(),
            "\"declined\""
        );

        let disabled: CustomerState = serde_json::from_str("\"disabled\"").unwrap();
        let enabled: CustomerState = serde_json::from_str("\"enabled\"").unwrap();

        assert_eq!(disabled, CustomerState::Disabled);
        assert_eq!(enabled, CustomerState::Enabled);
        assert_eq!(CustomerState::default(), CustomerState::Enabled);
    }

    #[test]
    fn test_customer_list_params_with_date_filters() {
        let created_at_min = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let params = CustomerListParams {
            ids: Some(vec![123, 456, 789]),
            limit: Some(50),
            since_id: Some(100),
            created_at_min: Some(created_at_min),
            created_at_max: None,
            updated_at_min: None,
            updated_at_max: None,
            fields: Some("id,email".into()),
            page_info: None,
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["ids"], serde_json::json!([123, 456, 789]));
        assert_eq!(json["limit"], 50);
        assert!(json["created_at_min"].as_str().is_some());

        let empty_params = CustomerListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_marketing_consent_structs() {
        let email_consent = EmailMarketingConsent {
            state: Some("subscribed".into()),
            opt_in_level: Some("single_opt_in".into()),
            consent_updated_at: None,
        };

        let email_json = serde_json::to_value(&email_consent).unwrap();
        assert_eq!(email_json["state"], "subscribed");
        assert_eq!(email_json["opt_in_level"], "single_opt_in");

        let sms_consent = SmsMarketingConsent {
            state: Some("subscribed".into()),
            opt_in_level: Some("single_opt_in".into()),
            consent_updated_at: None,
            consent_collected_from: Some("SHOPIFY".into()),
        };

        let sms_json = serde_json::to_value(&sms_consent).unwrap();
        assert_eq!(sms_json["consent_collected_from"], "SHOPIFY");

        assert_eq!(EmailMarketingConsent::default().state, None);
        assert_eq!(SmsMarketingConsent::default().consent_collected_from, None);
    }

    #[test]
    fn test_customer_get_id_returns_correct_value() {
        let customer_with_id = Customer {
            id: Some(123456789),
            email: Some("test@example.com".into()),
            ..Default::default()
        };
        assert_eq!(customer_with_id.get_id(), Some(123456789));

        let customer_without_id = Customer {
            id: None,
            email: Some("new@example.com".into()),
            ..Default::default()
        };
        assert_eq!(customer_without_id.get_id(), None);
    }

    #[test]
    fn test_customer_path_constants_are_correct() {
        let find_path = get_path(Customer::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "customers/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        let all_path = get_path(Customer::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "customers");

        let count_path = get_path(Customer::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "customers/count");

        let create_path = get_path(Customer::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        let update_path = get_path(Customer::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        let delete_path = get_path(Customer::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        assert_eq!(Customer::NAME, "Customer");
        assert_eq!(Customer::PLURAL, "customers");
    }
}
