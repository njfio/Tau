use tau_ai::Provider;
use tau_provider::{provider_auth_capability, provider_supported_auth_modes, ProviderAuthMethod};

#[test]
fn spec_c05_provider_auth_matrix_contract_is_complete_and_deterministic() {
    let providers = [
        Provider::OpenAi,
        Provider::OpenRouter,
        Provider::Anthropic,
        Provider::Google,
    ];
    let methods = [
        ProviderAuthMethod::ApiKey,
        ProviderAuthMethod::OauthToken,
        ProviderAuthMethod::Adc,
        ProviderAuthMethod::SessionToken,
    ];

    for provider in providers {
        let supported_modes = provider_supported_auth_modes(provider);
        for method in methods {
            let capability = provider_auth_capability(provider, method);
            assert_eq!(capability.method, method);
            assert!(
                !capability.reason.trim().is_empty(),
                "provider={provider:?} method={method:?} returned empty reason"
            );
            assert_eq!(
                supported_modes.contains(&method),
                capability.supported,
                "provider={provider:?} method={method:?} mismatch between capability and supported list"
            );
        }
    }
}

#[test]
fn conformance_provider_auth_matrix_matches_expected_support_by_provider() {
    let expected = [
        (
            Provider::OpenAi,
            [true, true, false, true], // api_key/oauth/adc/session
        ),
        (Provider::OpenRouter, [true, true, false, true]),
        (Provider::Anthropic, [true, true, false, true]),
        (Provider::Google, [true, true, true, false]),
    ];
    let methods = [
        ProviderAuthMethod::ApiKey,
        ProviderAuthMethod::OauthToken,
        ProviderAuthMethod::Adc,
        ProviderAuthMethod::SessionToken,
    ];

    for (provider, expectations) in expected {
        for (index, method) in methods.iter().enumerate() {
            let capability = provider_auth_capability(provider, *method);
            assert_eq!(
                capability.supported, expectations[index],
                "provider={provider:?} method={method:?} support drift"
            );
        }
    }
}
