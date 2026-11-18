//! Provider settings (jig/docs/design/domain/engine/llm-providers.md, 1–5).
use super::{Current, Edit, Failure, Fault, Item, Setting};
use alloc::boxed::Box;
pub use jig_domain_accounts::ProviderList;
use skein_lib::{Duration, List};
pub const PROVIDERS: u32 = 16;
pub const ENDPOINTS: u32 = 64;
pub const HEADERS: u32 = 16;
pub const PROGRAMS: u32 = 64;
pub const SCOPES: u32 = 64;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EndpointAddress {
    Socket(core::net::SocketAddr),
    Host { name: Box<[u8]>, port: u16 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProviderSetting {
    pub provider: Provider,
    pub endpoints: Box<[Endpoint]>,
    pub withdrawn: bool,
}
/// One configured LLM provider.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Provider {
    /// Its name for showing.
    pub name: Box<[u8]>,
    pub number: u32,
    /// The endpoint its added models take (configuration.md, 3).
    pub endpoint: Option<Box<[u8]>>,
    /// The program-free endpoint its sign-ins and keys take (configuration.md, 3).
    pub direct: Option<Box<[u8]>>,
    pub list: Option<ProviderList>,
    pub signed_in_plan: bool,
    pub manage_usage: Option<Box<[u8]>>,
    /// The sign-in it offers, with its public registration.
    pub sign_in: Option<Registration>,
    /// Whether it offers a key.
    pub keys: bool,
    /// The programs it offers, such as `claude_code`.
    pub programs: Box<[Box<[u8]>]>,
}

/// A provider's public registration for its sign-in: a client secret is refused.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Registration {
    pub issuer: Box<[u8]>,
    pub client: Box<[u8]>,
    pub scopes: Box<[Box<[u8]>]>,
    pub name_hint: Option<Box<[u8]>>,
    pub authorization: Box<[u8]>,
    pub token: Box<[u8]>,
    pub revocation: Option<Box<[u8]>>,
    pub keys: Box<[u8]>,
    pub endpoint_transport: Transport,
    pub must_grant: Box<[Box<[u8]>]>,
    pub resource: Option<Box<[u8]>>,
    pub redirect_path: Box<[u8]>,
    pub redirect_port: Option<u16>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Endpoint {
    pub name: Box<[u8]>,
    pub number: u32,
    pub dialect: u32,
    pub provider: Box<[u8]>,
    pub program: Option<Box<[u8]>>,
    pub address: EndpointAddress,
    pub transport: Transport,
    pub headers: Box<[Header]>,
    pub connect: Duration,
    pub handshake: Duration,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Header {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Transport {
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
    Plaintext,
}

fn failure(setting: Setting, fault: Fault) -> Failure {
    Failure { setting, fault, detail: None }
}
#[must_use]
pub fn program_name(name: &[u8]) -> bool {
    if name.is_empty() || name.len() > 32 {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' && *byte != b'_' {
            return false;
        }
    }
    true
}
#[must_use]
pub fn pattern(pattern: &[u8]) -> bool {
    if pattern.is_empty() || pattern.len() > 64 {
        return false;
    }
    for &byte in pattern {
        if byte != b'*' && byte != b'?' && !super::catalogue::valid_name(&[byte]) {
            return false;
        }
    }
    true
}
#[must_use]
pub fn scope_token(scope: &[u8]) -> bool {
    if scope.is_empty() || scope.len() > 4096 {
        return false;
    }
    for byte in scope {
        match byte {
            0x21 | 0x23..=0x5b | 0x5d..=0x7e => {}
            _ => return false,
        }
    }
    true
}
#[must_use]
pub fn public_text(value: &[u8], max: u32) -> bool {
    if value.is_empty() || value.len() > skein_lib::usize_of(max) || !skein_lib::bytes::is_utf8(value) {
        return false;
    }
    let mut before = 0_u8;
    let mut earlier = 0_u8;
    for byte in value {
        let format = (before == 0xd8 && *byte == 0x9c)
            || (before == 0xc2 && *byte == 0xad)
            || (earlier == 0xe2 && before == 0x80 && ((0x8b..=0x8f).contains(byte) || (0xaa..=0xae).contains(byte)))
            || (earlier == 0xe2 && before == 0x81 && (0xa0..=0xaf).contains(byte))
            || (earlier == 0xe1 && before == 0xa0 && *byte == 0x8e)
            || (earlier == 0xef && before == 0xbb && *byte == 0xbf)
            || (earlier == 0xef && before == 0xbf && (0xb9..=0xbb).contains(byte));
        if *byte < b' ' || *byte == 0x7f || (before == 0xc2 && (0x80..=0x9f).contains(byte)) || format {
            return false;
        }
        earlier = before;
        before = *byte;
    }
    true
}
#[must_use]
pub fn header(header: &Header) -> bool {
    for secret in [b"authorization".as_slice(), b"proxy-authorization", b"x-api-key", b"api-key", b"cookie"] {
        if header.name.eq_ignore_ascii_case(secret) {
            return false;
        }
    }
    !header.name.is_empty() && header.name.len() <= 4096 && header.value.len() <= 4096
}
fn transport(value: &Transport) -> bool {
    match value {
        Transport::Plaintext => true,
        Transport::Tls { server_name, trust } => {
            !server_name.is_empty()
                && server_name.len() <= 4096
                && match trust {
                    Some(path) => path.len() <= 4096,
                    None => true,
                }
        }
    }
}
#[must_use]
pub fn endpoint(value: &Endpoint) -> bool {
    if value.name.is_empty() || value.name.len() > 4096 || !transport(&value.transport) {
        return false;
    }
    if !dialect(value.dialect) {
        return false;
    }
    if let Some(program) = &value.program
        && !program_name(program)
    {
        return false;
    }
    let local = match &value.address {
        EndpointAddress::Socket(socket) => socket.ip().is_loopback(),
        EndpointAddress::Host { name, .. } => {
            if name.is_empty() || name.len() > 253 {
                return false;
            }
            false
        }
    };
    match value.transport {
        Transport::Plaintext if !local => return false,
        Transport::Plaintext | Transport::Tls { .. } => {}
    }
    if value.headers.len() > skein_lib::usize_of(HEADERS) {
        return false;
    }
    for item in &value.headers {
        if !header(item) {
            return false;
        }
    }
    true
}
fn registration(value: &Registration) -> bool {
    if value.scopes.len() > skein_lib::usize_of(SCOPES) || value.must_grant.len() > skein_lib::usize_of(SCOPES) {
        return false;
    }
    for text in [&value.name_hint, &value.revocation, &value.resource].into_iter().flatten() {
        if text.len() > 4096 {
            return false;
        }
    }
    for text in [&value.issuer, &value.client, &value.authorization, &value.token, &value.keys] {
        if text.is_empty() || text.len() > 4096 {
            return false;
        }
    }
    if value.redirect_path.len() > 4096
        || !redirect_path(&value.redirect_path)
        || value.redirect_port == Some(0)
        || !transport(&value.endpoint_transport)
    {
        return false;
    }
    for scope in &value.scopes {
        if !scope_token(scope) {
            return false;
        }
    }
    for scope in &value.must_grant {
        if !scope_token(scope) || !value.scopes.contains(scope) {
            return false;
        }
    }
    true
}
#[must_use]
pub fn fits(value: &ProviderSetting) -> bool {
    let provider = &value.provider;
    if provider.programs.len() > skein_lib::usize_of(PROGRAMS) {
        return false;
    }
    for text in [&provider.endpoint, &provider.direct].into_iter().flatten() {
        if text.len() > 4096 {
            return false;
        }
    }
    if provider.number == 0
        || !public_text(&provider.name, jig_domain_accounts::PROVIDER_NAME)
        || terms(provider).is_some()
    {
        return false;
    }
    if let Some(address) = &provider.manage_usage
        && (address.len() > skein_lib::usize_of(jig_domain_accounts::MANAGE_USAGE) || !public_address(address))
    {
        return false;
    }
    if let Some(value) = &provider.sign_in
        && !registration(value)
    {
        return false;
    }
    for (at, name) in provider.programs.iter().enumerate() {
        if !program_name(name) {
            return false;
        }
        for previous in provider.programs.iter().take(at) {
            if previous == name {
                return false;
            }
        }
    }
    if let Some(list) = &provider.list {
        if provider.endpoint.is_none() || list.keep.len() > 16 || list.drop.len() > 16 {
            return false;
        }
        for item in list.keep.iter().chain(list.drop.iter()) {
            if !pattern(item) {
                return false;
            }
        }
    }
    for (at, item) in value.endpoints.iter().enumerate() {
        if !endpoint(item) || item.provider != provider.name {
            return false;
        }
        if !program(provider, item) {
            return false;
        }
        for previous in value.endpoints.iter().take(at) {
            if previous.name == item.name || previous.number == item.number {
                return false;
            }
        }
    }
    for name in [&provider.endpoint, &provider.direct].into_iter().flatten() {
        let mut found = false;
        for item in &value.endpoints {
            found |= item.name == *name;
        }
        if !found {
            return false;
        }
    }
    direct(provider, &value.endpoints)
}

#[must_use]
pub fn direct(provider: &Provider, endpoints: &[Endpoint]) -> bool {
    if let Some(name) = &provider.direct {
        let mut dialect = None;
        for item in endpoints {
            if provider.endpoint.as_deref() == Some(item.name.as_ref()) && item.program.is_some() {
                dialect = Some(item.dialect);
            }
        }
        let mut valid = false;
        for item in endpoints {
            valid |= item.name == *name && item.program.is_none() && dialect == Some(item.dialect);
        }
        if !valid || (!provider.keys && provider.sign_in.is_none()) {
            return false;
        }
    }
    true
}
fn pair(setting: Setting, value: &ProviderSetting, held: Setting, previous: &ProviderSetting) -> Result<(), Failure> {
    if (held == setting) != (previous.provider.name == value.provider.name) {
        return Err(failure(setting, Fault::Shape));
    }
    Ok(())
}
fn identity(current: &Current, setting: Setting, value: &ProviderSetting) -> Result<(), Failure> {
    for (&held, default) in &current.defaults {
        if let Some(previous) = default.value.as_provider() {
            pair(setting, value, held, previous)?;
        }
    }
    for (&held, changed) in &current.changes {
        if let Some(stored) = &changed.value
            && let Some(previous) = stored.as_provider()
        {
            pair(setting, value, held, previous)?;
        }
    }
    for previous in &current.fixed_providers {
        pair(setting, value, Setting::Provider(previous.provider.number), previous)?;
    }
    Ok(())
}
pub fn check_change(current: &Current, items: &[Item]) -> Result<(), Failure> {
    for item in items {
        let number = match item.setting {
            Setting::Provider(number) => number,
            Setting::Worker(_)
            | Setting::Model(_)
            | Setting::Charter(_)
            | Setting::Template(_)
            | Setting::Policy(_)
            | Setting::Rules
            | Setting::Application { .. }
            | Setting::ProjectPolicy { .. } => continue,
        };
        if number == 0 {
            return Err(failure(item.setting, Fault::Shape));
        }
        if number > PROVIDERS {
            return Err(failure(item.setting, Fault::Count));
        }
        if fixed(current, number) {
            return Err(failure(item.setting, Fault::Shape));
        }
        match &item.edit {
            Edit::Set(value) => {
                let value = value.as_provider().ok_or(failure(item.setting, Fault::Shape))?;
                if value.withdrawn || !fits(value) {
                    return Err(failure(item.setting, Fault::Shape));
                }
                identity(current, item.setting, value)?;
            }
            Edit::Reset if current.default_value(item.setting).is_none() => {
                return Err(failure(item.setting, Fault::Shape));
            }
            Edit::Reset | Edit::Removed => {}
        }
    }
    Ok(())
}
#[must_use]
pub fn overlay(current: &Current, defaults: bool, items: &[Item]) -> Box<[ProviderSetting]> {
    let mut result = List::with_capacity(
        PROVIDERS.saturating_add(u32::try_from(current.fixed_providers.len()).expect("fixed provider count")),
    );
    for number in 1..=PROVIDERS {
        if let Some(value) = super::check::overlay_value(current, defaults, items, Setting::Provider(number))
            && let Some(value) = value.as_provider()
            && !value.withdrawn
        {
            result.push(value.clone()).expect("provider number bound");
        }
    }
    for value in &current.fixed_providers {
        if !value.withdrawn {
            result.push(value.clone()).expect("fixed provider count");
        }
    }
    result.into_boxed()
}
fn all<'a>(current: &'a Current, defaults: bool, items: &'a [Item]) -> List<&'a ProviderSetting> {
    let mut result = List::with_capacity(
        PROVIDERS.saturating_add(u32::try_from(current.fixed_providers.len()).expect("fixed count")),
    );
    for number in 1..=PROVIDERS {
        let setting = Setting::Provider(number);
        let value = match super::check::edit(items, setting) {
            Some(Edit::Removed) => {
                if defaults {
                    current.default_value(setting)
                } else {
                    current.value(setting)
                }
            }
            Some(Edit::Set(_) | Edit::Reset) | None => super::check::overlay_value(current, defaults, items, setting),
        };
        if let Some(value) = value
            && let Some(value) = value.as_provider()
        {
            result.push(value).expect("provider count");
        }
    }
    for value in &current.fixed_providers {
        result.push(value).expect("fixed count");
    }
    result
}
pub fn check_set(current: &Current, defaults: bool, items: &[Item]) -> Result<(), Failure> {
    for value in &current.fixed_providers {
        if value.withdrawn {
            return Err(failure(Setting::Provider(value.provider.number), Fault::Shape));
        }
    }
    let values = all(current, defaults, items);
    let mut endpoints = 0_u32;
    for (at, value) in values.iter().enumerate() {
        let setting = Setting::Provider(value.provider.number);
        endpoints = endpoints.saturating_add(u32::try_from(value.endpoints.len()).unwrap_or(u32::MAX));
        if value.provider.number > PROVIDERS || endpoints > ENDPOINTS {
            return Err(failure(setting, Fault::Count));
        }
        if !fits(value) {
            return Err(failure(setting, Fault::Shape));
        }
        identity(current, setting, value)?;
        for previous in values.iter().take(at) {
            if !distinct(value, previous) {
                return Err(failure(setting, Fault::Shape));
            }
        }
    }
    Ok(())
}
/// Heap bound for every provider's own fields and the shared endpoint count (section 2).
#[must_use]
pub fn set_bound() -> Option<u64> {
    let boxed = u64::try_from(size_of::<Box<[u8]>>()).ok()?;
    let header = u64::try_from(size_of::<Header>()).ok()?.checked_add(8192)?;
    let endpoint = u64::try_from(size_of::<Endpoint>())
        .ok()?
        .checked_add(4096 + 128 + 32 + 253 + 8192)?
        .checked_add(u64::from(HEADERS).checked_mul(header)?)?;
    let scopes = u64::from(SCOPES).checked_mul(boxed.checked_add(4096)?)?.checked_mul(2)?;
    let registration = scopes.checked_add(5 * 4096 + 3 * 4096 + 4096 + 8192)?;
    let provider = u64::try_from(size_of::<ProviderSetting>())
        .ok()?
        .checked_add(128 + 8192 + 2048)?
        .checked_add(u64::from(PROGRAMS).checked_mul(boxed.checked_add(32)?)?)?
        .checked_add(32_u64.checked_mul(boxed.checked_add(64)?)?)?
        .checked_add(registration)?;
    u64::from(PROVIDERS).checked_mul(provider)?.checked_add(u64::from(ENDPOINTS).checked_mul(endpoint)?)
}

/// Public HTTPS address with an ASCII authority and no credentials, query or fragment.
pub fn public_address(value: &[u8]) -> bool {
    let Some(rest) = value.strip_prefix(b"https://") else {
        return false;
    };
    for byte in rest {
        if *byte <= b' ' || *byte >= 127 || b"@?#\\\"'<>".contains(byte) {
            return false;
        }
    }
    let end = match skein_lib::bytes::find(rest, b"/") {
        Some(end) => end,
        None => rest.len(),
    };
    public_authority(rest.get(..end).expect("address authority prefix"))
}

#[derive(Clone, Copy, Debug)]
enum LabelByte {
    Label,
    Dash,
    Dot,
    Other,
}

#[expect(clippy::disallowed_methods, reason = "configuration parses an address's IPv6 literal (configuration.md, 3)")]
pub fn public_authority(authority: &[u8]) -> bool {
    if let Some(rest) = authority.strip_prefix(b"[") {
        let Some(end) = skein_lib::bytes::find(rest, b"]") else {
            return false;
        };
        let host = rest.get(..end).expect("IPv6 host prefix");
        let Ok(host) = core::str::from_utf8(host) else {
            return false;
        };
        if host.parse::<core::net::Ipv6Addr>().is_err() {
            return false;
        }
        return public_port(rest.get(end.saturating_add(1)..).expect("suffix after bracket"));
    }
    let end = match skein_lib::bytes::find(authority, b":") {
        Some(end) => end,
        None => authority.len(),
    };
    let host = authority.get(..end).expect("host prefix");
    let mut label = 0_u32;
    let mut dash = false;
    for byte in host {
        let class = if byte.is_ascii_alphanumeric() {
            LabelByte::Label
        } else if *byte == b'-' {
            LabelByte::Dash
        } else if *byte == b'.' {
            LabelByte::Dot
        } else {
            LabelByte::Other
        };
        match class {
            LabelByte::Label => {
                label = label.saturating_add(1);
                dash = false;
            }
            LabelByte::Dash => {
                if label == 0 {
                    return false;
                }
                label = label.saturating_add(1);
                dash = true;
            }
            LabelByte::Dot => {
                if label == 0 || dash {
                    return false;
                }
                label = 0;
            }
            LabelByte::Other => return false,
        }
        if label > 63 {
            return false;
        }
    }
    label > 0 && !dash && public_port(authority.get(end..).expect("port suffix"))
}
fn public_port(suffix: &[u8]) -> bool {
    if suffix.is_empty() {
        return true;
    }
    let Some(port) = suffix.strip_prefix(b":") else {
        return false;
    };
    let mut value = 0_u32;
    for byte in port {
        if !byte.is_ascii_digit() {
            return false;
        }
        let digit = u32::from(byte.checked_sub(b'0').expect("ASCII digit"));
        let Some(next) = value.checked_mul(10) else {
            return false;
        };
        let Some(next) = next.checked_add(digit) else {
            return false;
        };
        value = next;
    }
    (1..=65_535).contains(&value)
}

/// Names and endpoint identities are unique across providers, including withdrawn ones.
#[must_use]
pub fn distinct(value: &ProviderSetting, previous: &ProviderSetting) -> bool {
    if value.provider.number == previous.provider.number || value.provider.name == previous.provider.name {
        return false;
    }
    for endpoint in &value.endpoints {
        for other in &previous.endpoints {
            if endpoint.number == other.number || endpoint.name == other.name {
                return false;
            }
        }
    }
    true
}

/// The heap retained by one provider value, including its value carrier.
#[must_use]
pub fn owned_bytes(value: &ProviderSetting) -> Option<u64> {
    let provider = &value.provider;
    let mut bytes =
        u64::try_from(size_of::<ProviderSetting>()).ok()?.checked_add(skein_lib::len_u64(provider.name.len()))?;
    for text in [&provider.endpoint, &provider.direct, &provider.manage_usage].into_iter().flatten() {
        bytes = bytes.checked_add(skein_lib::len_u64(text.len()))?;
    }
    bytes = bytes.checked_add(texts_bytes(&provider.programs)?)?;
    if let Some(list) = &provider.list {
        bytes = bytes.checked_add(texts_bytes(&list.keep)?)?.checked_add(texts_bytes(&list.drop)?)?;
    }
    if let Some(registration) = &provider.sign_in {
        for text in [
            &registration.issuer,
            &registration.client,
            &registration.authorization,
            &registration.token,
            &registration.keys,
            &registration.redirect_path,
        ] {
            bytes = bytes.checked_add(skein_lib::len_u64(text.len()))?;
        }
        for text in [&registration.name_hint, &registration.revocation, &registration.resource].into_iter().flatten() {
            bytes = bytes.checked_add(skein_lib::len_u64(text.len()))?;
        }
        bytes = bytes
            .checked_add(texts_bytes(&registration.scopes)?)?
            .checked_add(texts_bytes(&registration.must_grant)?)?
            .checked_add(transport_bytes(&registration.endpoint_transport)?)?;
    }
    bytes = bytes.checked_add(
        skein_lib::len_u64(value.endpoints.len()).checked_mul(u64::try_from(size_of::<Endpoint>()).ok()?)?,
    )?;
    for endpoint in &value.endpoints {
        bytes = bytes
            .checked_add(skein_lib::len_u64(endpoint.name.len()))?
            .checked_add(skein_lib::len_u64(endpoint.provider.len()))?
            .checked_add(transport_bytes(&endpoint.transport)?)?;
        if let Some(program) = &endpoint.program {
            bytes = bytes.checked_add(skein_lib::len_u64(program.len()))?;
        }
        match &endpoint.address {
            EndpointAddress::Socket(_) => {}
            EndpointAddress::Host { name, .. } => bytes = bytes.checked_add(skein_lib::len_u64(name.len()))?,
        }
        bytes = bytes.checked_add(
            skein_lib::len_u64(endpoint.headers.len()).checked_mul(u64::try_from(size_of::<Header>()).ok()?)?,
        )?;
        for header in &endpoint.headers {
            bytes = bytes
                .checked_add(skein_lib::len_u64(header.name.len()))?
                .checked_add(skein_lib::len_u64(header.value.len()))?;
        }
    }
    Some(bytes)
}
fn transport_bytes(value: &Transport) -> Option<u64> {
    match value {
        Transport::Plaintext => Some(0),
        Transport::Tls { server_name, trust } => skein_lib::len_u64(server_name.len()).checked_add(match trust {
            Some(value) => skein_lib::len_u64(value.len()),
            None => 0,
        }),
    }
}
fn texts_bytes(values: &[Box<[u8]>]) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(values.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
    for value in values {
        bytes = bytes.checked_add(skein_lib::len_u64(value.len()))?;
    }
    Some(bytes)
}

#[must_use]
pub fn fixed(current: &Current, number: u32) -> bool {
    for provider in &current.fixed_providers {
        if provider.provider.number == number {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests;

#[must_use]
pub fn dialect(number: u32) -> bool {
    number == 2 || number == 3
}
#[must_use]
pub fn redirect_path(value: &[u8]) -> bool {
    !value.is_empty() && !value.starts_with(b"/") && !value.contains(&b'?') && !value.contains(&b'#')
}
#[derive(Clone, Copy, Debug)]
pub enum TermsFault {
    SignedInPlan,
    ManageUsage,
}
#[must_use]
pub fn terms(provider: &Provider) -> Option<TermsFault> {
    if provider.signed_in_plan && provider.sign_in.is_none() {
        return Some(TermsFault::SignedInPlan);
    }
    if (provider.signed_in_plan || !provider.programs.is_empty()) && provider.manage_usage.is_none() {
        return Some(TermsFault::ManageUsage);
    }
    None
}
#[must_use]
pub fn program(provider: &Provider, endpoint: &Endpoint) -> bool {
    match &endpoint.program {
        Some(name) => program_name(name) && provider.programs.contains(name),
        None => true,
    }
}

/// The program used by a document model must be offered by its provider.
#[must_use]
pub fn program_bindings(endpoints: &[Endpoint], models: &[super::CatalogueModel], providers: &[Provider]) -> List<u32> {
    let mut invalid = List::with_capacity(u32::try_from(endpoints.len()).expect("endpoint count"));
    for (index, endpoint) in endpoints.iter().enumerate() {
        if endpoint.program.is_none() {
            continue;
        }
        let mut valid = false;
        let mut refused = false;
        for provider in providers {
            if provider.endpoint.as_deref() == Some(endpoint.name.as_ref()) {
                valid = true;
                if !program(provider, endpoint) {
                    refused = true;
                }
            }
        }
        for model in models {
            if model.model.endpoint != endpoint.number {
                continue;
            }
            for provider in providers {
                if provider.number == model.model.provider {
                    valid = true;
                    if !program(provider, endpoint) {
                        refused = true;
                    }
                }
            }
        }
        if !valid || refused {
            invalid.push(u32::try_from(index).expect("endpoint count")).expect("one problem per endpoint");
        }
    }
    invalid
}
