#[cfg(test)]
mod tests {
    use garden::api::primitives::{Response, Status};
    use http_provider_macro::http_provider;
    use reqwest::{header::HeaderMap, Url};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    // Define the provider and its methods using the macro
    http_provider!(
        HttpProvider,
        {
            {
                path: "/custom-path",
                method: GET,
                fn_name: fetch_a,
                req: MyRequest,
                res: garden::api::primitives::Response<MyResponse>,
                headers : reqwest::header::HeaderMap,
                query_params : MyQueryParams,
            },
                 {
                path: "/custom-path",
                method: POST,
                fn_name: post_b,
                req: MyRequest,
                res: garden::api::primitives::Response<MyResponse>,
            },
            {
                path: "/custom-path",
                method: PUT,
                fn_name: put_c,
                req: MyRequest,
                res: garden::api::primitives::Response<MyResponse>,
            },
            {
                path: "/custom-path",
                method: DELETE,
                fn_name: delete_d,
                res: garden::api::primitives::Response<MyResponse>,
            },
            {
                path: "/custom-path/{id}",
                method: GET,
                fn_name: get_user_by_id,
                path_params: MyPathParams,
                res: garden::api::primitives::Response<MyResponse>,
            }
        }

    );

    #[derive(Serialize, Deserialize)]
    struct MyPathParams {
        id: String,
    }

    #[derive(Serialize, Deserialize)]
    struct MyQueryParams {
        query: String,
    }

    // Define the request and response types
    #[derive(Serialize, Deserialize)]
    struct MyRequest {
        query: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct MyResponse {
        value: String,
    }

    #[tokio::test]
    async fn test_successful_get_response() -> Result<(), Box<dyn std::error::Error>> {
        use wiremock::matchers::{header, method, query_param};

        // Start the mock server
        let mock_server = MockServer::start().await;

        // Define expected response
        let response = Response::<MyResponse> {
            status: Status::Ok,
            result: Some(MyResponse {
                value: "Hello world".to_string(),
            }),
            error: None,
        };

        // Set up a mock that checks for the query and a custom header
        Mock::given(method("GET"))
            .and(query_param("query", "Helo")) // check query param
            .and(header("x-custom-header", "myvalue")) // check header
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let url = Url::from_str(&mock_server.uri())?;
        let provider = HttpProvider::new(url, 5);

        // Create headers with a custom value
        let mut headers = HeaderMap::new();
        headers.insert("x-custom-header", "myvalue".parse()?);

        // Call the GET method
        let result = provider
            .fetch_a(
                &MyRequest {
                    query: "Helo".to_string(),
                },
                headers,
                MyQueryParams {
                    query: "Helo".to_string(),
                },
            )
            .await?;

        println!("Result : {:#?}", result);

        // Assert expected response
        assert_eq!(result.status, Status::Ok);
        assert_eq!(
            result.result,
            Some(MyResponse {
                value: "Hello world".to_string()
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_path_param_get_user_by_id() -> Result<(), Box<dyn std::error::Error>> {
        use wiremock::matchers::{method, path_regex};

        // Start the mock server
        let mock_server = MockServer::start().await;

        // Define expected response
        let response = Response::<MyResponse> {
            status: Status::Ok,
            result: Some(MyResponse {
                value: "User42".to_string(),
            }),
            error: None,
        };

        // Set up a mock that matches the dynamic path
        Mock::given(method("GET"))
            .and(path_regex(r"^/custom-path/\w+$")) // Accepts any /users/{id}
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let url = Url::from_str(&mock_server.uri())?;

        let provider = HttpProvider::new(url, 5);

        // Call the generated GET method with path params
        let path_params = MyPathParams {
            id: "42".to_string(),
        };

        let result = provider.get_user_by_id(&path_params).await?;

        println!("Result : {:#?}", result);

        // Assert expected response
        assert_eq!(result.status, Status::Ok);
        assert_eq!(
            result.result,
            Some(MyResponse {
                value: "User42".to_string()
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_successful_post_response() -> Result<(), Box<dyn std::error::Error>> {
        // Start the mock server
        let mock_server = MockServer::start().await;

        // Set up a mock response for the POST method
        let response = garden::api::primitives::Response::<MyResponse> {
            status: garden::api::primitives::Status::Ok,
            result: Some(MyResponse {
                value: "Post success".to_string(),
            }),
            error: None,
        };

        // Define how the mock server should respond to a POST request at /orderbook/b
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        // URL to use for the POST request (from the mock server)
        let url = Url::from_str(&mock_server.uri())?;

        // Instantiate the provider (using the macro-generated OrderbookProvider)
        let provider = HttpProvider::new(url, 5);

        // Prepare the request body
        let req = MyRequest {
            query: "test".to_string(),
        };

        // Call the POST function using the mock server
        let result = provider.post_b(&req).await?;

        println!("Result: {:#?}", result);

        // Assert that the response matches the expected value
        assert_eq!(result.status, garden::api::primitives::Status::Ok);
        assert_eq!(
            result.result,
            Some(MyResponse {
                value: "Post success".to_string()
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_successful_put_response() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = wiremock::MockServer::start().await;

        let response = garden::api::primitives::Response::<MyResponse> {
            status: garden::api::primitives::Status::Ok,
            result: Some(MyResponse {
                value: "Put success".to_string(),
            }),
            error: None,
        };

        wiremock::Mock::given(wiremock::matchers::method("PUT"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let url = reqwest::Url::from_str(&mock_server.uri())?;
        let provider = HttpProvider::new(url, 5);

        let req = MyRequest {
            query: "test put".to_string(),
        };

        let result = provider.put_c(&req).await?;
        assert_eq!(result.status, garden::api::primitives::Status::Ok);
        assert_eq!(
            result.result,
            Some(MyResponse {
                value: "Put success".to_string()
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_successful_delete_response() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = wiremock::MockServer::start().await;

        let response = garden::api::primitives::Response::<MyResponse> {
            status: garden::api::primitives::Status::Ok,
            result: Some(MyResponse {
                value: "Delete success".to_string(),
            }),
            error: None,
        };

        wiremock::Mock::given(wiremock::matchers::method("DELETE"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(response))
            .mount(&mock_server)
            .await;

        let url = reqwest::Url::from_str(&mock_server.uri())?;
        let provider = HttpProvider::new(url, 5);

        let result = provider.delete_d().await?;

        assert_eq!(result.status, garden::api::primitives::Status::Ok);
        assert_eq!(
            result.result,
            Some(MyResponse {
                value: "Delete success".to_string()
            })
        );
        Ok(())
    }
}
