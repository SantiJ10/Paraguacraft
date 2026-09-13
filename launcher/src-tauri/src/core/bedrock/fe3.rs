//! Cliente SOAP mínimo de Microsoft FE3 (Windows Update delivery).
//!
//! Port del flujo anónimo `GetExtendedUpdateInfo2` de XMCL / mc-w10-version-launcher.
//! Solo alcanza para releases públicas; beta/preview piden ticket MSA.

use crate::error::{AppError, AppResult};

const SECURED_URL: &str = "https://fe3.delivery.mp.microsoft.com/ClientWebService/client.asmx/secured";
const DELIVERY_HOST_PREFIX: &str = "http://tlu.dl.delivery.mp.microsoft.com/";

const DEVICE_ATTRIBUTES: &str = "E:BranchReadinessLevel=CBB&DchuNvidiaGrfxExists=1&ProcessorIdentifier=Intel64%20Family%206%20Model%2063%20Stepping%202&CurrentBranch=rs4_release&DataVer_RS5=1942&FlightRing=Retail&AttrDataVer=57&InstallLanguage=en-US&DchuAmdGrfxExists=1&OSUILocale=en-US&InstallationType=Client&FlightingBranchName=&Version_RS5=10&UpgEx_RS5=Green&GStatus_RS5=2&OSSkuId=48&App=WU&InstallDate=1529700913&ProcessorManufacturer=GenuineIntel&AppVer=10.0.17134.471&OSArchitecture=AMD64&UpdateManagementGroup=2&IsDeviceRetailDemo=0&HidOverGattReg=C%3A%5CWINDOWS%5CSystem32%5CDriverStore%5CFileRepository%5Chidbthle.inf_amd64_467f181075371c89%5CMicrosoft.Bluetooth.Profiles.HidOverGatt.dll&IsFlightingEnabled=0&DchuIntelGrfxExists=1&TelemetryLevel=1&DefaultUserRegion=244&DeferFeatureUpdatePeriodInDays=365&Bios=Unknown&WuClientVer=10.0.17134.471&PausedFeatureStatus=1&Steam=URL%3Asteam%20protocol&Free=8to16&OSVersion=10.0.17134.472&DeviceFamily=Windows.Desktop";

pub fn is_guid(value: &str) -> bool {
    if value.len() != 36 {
        return false;
    }
    value.as_bytes().iter().enumerate().all(|(i, &b)| match i {
        8 | 13 | 18 | 23 => b == b'-',
        _ => b.is_ascii_hexdigit(),
    })
}

fn xml_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn xml_unescape(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn unix_to_iso(secs: u64) -> String {
    let days = (secs / 86400) as i64;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}.000Z")
}

/// Howard Hinnant: days since Unix epoch → civil date.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

fn now_created_expires() -> (String, String) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (unix_to_iso(now), unix_to_iso(now.saturating_add(300)))
}

fn build_download_request(update_identity: &str, revision_number: &str) -> String {
    let (created, expires) = now_created_expires();
    let id = xml_escape(update_identity);
    let rev = xml_escape(revision_number);
    let attrs = xml_escape(DEVICE_ATTRIBUTES);
    format!(
        r#"<s:Envelope xmlns:a="http://www.w3.org/2005/08/addressing" xmlns:s="http://www.w3.org/2003/05/soap-envelope">
  <s:Header>
    <a:Action s:mustUnderstand="1">http://www.microsoft.com/SoftwareDistribution/Server/ClientWebService/GetExtendedUpdateInfo2</a:Action>
    <a:MessageID>urn:uuid:5754a03d-d8d5-489f-b24d-efc31b3fd32d</a:MessageID>
    <a:To s:mustUnderstand="1">{SECURED_URL}</a:To>
    <o:Security s:mustUnderstand="1" xmlns:o="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd">
      <Timestamp xmlns="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd">
        <Created>{created}</Created>
        <Expires>{expires}</Expires>
      </Timestamp>
      <wuws:WindowsUpdateTicketsToken wsu:id="ClientMSA" xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd" xmlns:wuws="http://schemas.microsoft.com/msus/2014/10/WindowsUpdateAuthorization">
        <TicketType Name="AAD" Version="1.0" Policy="MBI_SSL"></TicketType>
      </wuws:WindowsUpdateTicketsToken>
    </o:Security>
  </s:Header>
  <s:Body>
    <GetExtendedUpdateInfo2 xmlns="http://www.microsoft.com/SoftwareDistribution/Server/ClientWebService">
      <updateIDs>
        <UpdateIdentity>
          <UpdateID>{id}</UpdateID>
          <RevisionNumber>{rev}</RevisionNumber>
        </UpdateIdentity>
      </updateIDs>
      <infoTypes>
        <XmlUpdateFragmentType>FileUrl</XmlUpdateFragmentType>
      </infoTypes>
      <deviceAttributes>{attrs}</deviceAttributes>
    </GetExtendedUpdateInfo2>
  </s:Body>
</s:Envelope>"#
    )
}

fn extract_download_urls(response_xml: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut rest = response_xml;
    while let Some(start) = rest.find("<Url>") {
        rest = &rest[start + 5..];
        let Some(end) = rest.find("</Url>") else { break };
        let raw = rest[..end].trim();
        if !raw.is_empty() {
            urls.push(xml_unescape(raw));
        }
        rest = &rest[end + 6..];
    }
    if urls.is_empty() {
        rest = response_xml;
        while let Some(start) = rest.find("<wu:Url>") {
            rest = &rest[start + 8..];
            let Some(end) = rest.find("</wu:Url>") else { break };
            let raw = rest[..end].trim();
            if !raw.is_empty() {
                urls.push(xml_unescape(raw));
            }
            rest = &rest[end + 9..];
        }
    }
    urls
}

pub fn pick_cdn_url(urls: &[String]) -> Option<String> {
    urls.iter()
        .find(|u| u.starts_with(DELIVERY_HOST_PREFIX))
        .cloned()
}

/// Resuelve la URL directa del AppX en el CDN oficial de Microsoft.
pub async fn resolve_download_url(
    client: &reqwest::Client,
    update_identity: &str,
    revision_number: &str,
) -> AppResult<String> {
    let identity = update_identity.trim();
    if identity.starts_with("http://") || identity.starts_with("https://") {
        return Ok(identity.to_string());
    }
    if !is_guid(identity) {
        return Err(AppError::msg(format!(
            "Identidad de actualización Bedrock inválida: {identity}"
        )));
    }
    let body = build_download_request(identity, revision_number);
    let resp = client
        .post(SECURED_URL)
        .header("content-type", "application/soap+xml; charset=utf-8")
        .header("user-agent", "Windows-Update-Agent/10.0.10011.16384 Client-Protocol/2.0")
        .body(body)
        .send()
        .await
        .map_err(|e| {
            AppError::msg(format!(
                "No se pudo contactar el CDN de Microsoft (FE3): {e}"
            ))
        })?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::msg(format!(
            "FE3 falló ({status}): {}",
            text.chars().take(280).collect::<String>()
        )));
    }
    let urls = extract_download_urls(&text);
    pick_cdn_url(&urls).ok_or_else(|| {
        AppError::msg(
            "No hay URL pública para esta versión. Las betas/preview requieren el programa Insider.",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guid_shape() {
        assert!(is_guid("5754a03d-d8d5-489f-b24d-efc31b3fd32d"));
        assert!(!is_guid("not-a-guid"));
        assert!(!is_guid("5754a03d-d8d5-489f-b24d-efc31b3fd32"));
    }

    #[test]
    fn xml_roundtrip_amp() {
        assert_eq!(xml_escape("a&b<c>"), "a&amp;b&lt;c&gt;");
        assert_eq!(xml_unescape("a&amp;b&lt;c&gt;"), "a&b<c>");
    }

    #[test]
    fn picks_official_cdn() {
        let urls = vec![
            "https://example.com/nope".into(),
            "http://tlu.dl.delivery.mp.microsoft.com/filestreamingservice/files/abc".into(),
        ];
        assert!(pick_cdn_url(&urls)
            .unwrap()
            .starts_with(DELIVERY_HOST_PREFIX));
    }

    #[test]
    fn extracts_url_nodes() {
        let xml = r#"<FileLocation><Url>http://tlu.dl.delivery.mp.microsoft.com/x</Url></FileLocation>"#;
        assert_eq!(
            extract_download_urls(xml),
            vec!["http://tlu.dl.delivery.mp.microsoft.com/x"]
        );
    }

    #[test]
    fn unix_epoch_iso() {
        assert_eq!(unix_to_iso(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(unix_to_iso(1_704_067_200), "2024-01-01T00:00:00.000Z");
    }
}
