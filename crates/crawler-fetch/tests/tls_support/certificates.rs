//! Generates independent certificate authorities and server keys entirely in memory.

use rcgen::{BasicConstraints, CertificateParams, CertifiedIssuer, IsCa, KeyPair, KeyUsagePurpose};
use rustls::ServerConfig;
use rustls::pki_types::PrivatePkcs8KeyDer;

pub type BoxError = Box<dyn std::error::Error>;

pub struct Certificates {
    o_issuer: CertifiedIssuer<'static, KeyPair>,
}

impl Certificates {
    pub fn new() -> Result<Self, BoxError> {
        let o_key = KeyPair::generate()?;
        let mut o_params = CertificateParams::new(Vec::<String>::new())?;
        let by_id = o_key.public_key_raw().get(..8).ok_or("root key length")?;
        o_params.distinguished_name.push(
            rcgen::DnType::CommonName,
            format!("proofcrawl {:02x?}", by_id),
        );
        o_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        o_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        let o_issuer = CertifiedIssuer::self_signed(o_params, o_key)?;
        Ok(Self { o_issuer })
    }

    pub fn root(&self) -> Result<reqwest::Certificate, BoxError> {
        Ok(reqwest::Certificate::from_der(self.o_issuer.der())?)
    }

    pub fn server(&self, str_host: &str) -> Result<ServerConfig, BoxError> {
        let o_key = KeyPair::generate()?;
        let mut o_params = CertificateParams::new(vec![str_host.to_owned()])?;
        o_params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        let o_leaf = o_params.signed_by(&o_key, &self.o_issuer)?;
        let o_key = PrivatePkcs8KeyDer::from(o_key.serialize_der());
        Ok(ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![o_leaf.der().clone()], o_key.into())?)
    }
}
