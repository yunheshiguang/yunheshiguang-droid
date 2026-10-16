#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceKind {
    Connect,
    Pairing,
    Legacy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MdnsService {
    pub instance: String,
    pub kind: ServiceKind,
    pub host: String,
    pub port: u32,
}

impl MdnsService {
    pub fn is_attached(&self, serials: &[String]) -> bool {
        serials.iter().any(|serial| {
            serial.starts_with(&format!("{}.", self.instance))
                || *serial == format!("{}:{}", self.host, self.port)
        })
    }
}

fn kind_of(service_type: &str) -> Option<ServiceKind> {
    match service_type.trim_end_matches('.') {
        "_adb-tls-connect._tcp" => Some(ServiceKind::Connect),
        "_adb-tls-pairing._tcp" => Some(ServiceKind::Pairing),
        "_adb._tcp" => Some(ServiceKind::Legacy),
        _ => None,
    }
}

fn parse_line(line: &str) -> Option<MdnsService> {
    let fields = if line.contains('\t') {
        line.split('\t')
            .map(str::trim)
            .filter(|field| !field.is_empty())
            .collect::<Vec<_>>()
    } else {
        line.split_whitespace().collect::<Vec<_>>()
    };
    let [instance, service_type, .., address] = fields.as_slice() else {
        return None;
    };
    let kind = kind_of(service_type)?;
    let (host, port) = address.rsplit_once(':')?;

    Some(MdnsService {
        instance: (*instance).to_string(),
        kind,
        host: host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string(),
        port: port.parse().ok()?,
    })
}

pub fn parse_services(stdout: &[u8]) -> Vec<MdnsService> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter_map(parse_line)
        .collect()
}
