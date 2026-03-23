use rkyv::{Archive, Deserialize, Serialize, vec::ArchivedVec};

#[derive(
    serde::Serialize, serde::Deserialize, Debug, PartialEq, Archive, Serialize, Deserialize,
)]
pub struct HgncRecord {
    pub hgnc_id: String,
    pub symbol: String,
    pub name: String,
    pub locus_group: String,
    pub locus_type: String,
    pub status: String,
    pub location: String,
    pub location_sortable: String,
    pub alias_symbol: String,
    pub alias_name: String,
    pub prev_symbol: String,
    pub prev_name: String,
    pub gene_group: String,
    pub gene_group_id: String,
    pub date_approved_reserved: String,
    pub date_symbol_changed: String,
    pub date_name_changed: String,
    pub date_modified: String,
    pub entrez_id: String,
    pub ensembl_gene_id: String,
    pub vega_id: String,
    pub ucsc_id: String,
    pub ena: String,
    pub refseq_accession: String,
    pub ccds_id: String,
    pub uniprot_ids: String,
    pub pubmed_id: String,
    pub mgd_id: String,
    pub rgd_id: String,
    pub lsdb: String,
    pub cosmic: String,
    pub omim_id: String,
    pub mirbase: String,
    pub homeodb: String,
    pub snornabase: String,
    pub bioparadigms_slc: String,
    pub orphanet: String,
    #[serde(rename = "pseudogene.org")]
    pub pseudogene_org: String,
    pub horde_id: String,
    pub merops: String,
    pub imgt: String,
    pub iuphar: String,
    pub kznf_gene_catalog: String,
    #[serde(rename = "mamit-trnadb")]
    pub mamit_trnadb: String,
    pub cd: String,
    pub lncrnadb: String,
    pub enzyme_id: String,
    pub intermediate_filament_db: String,
    pub rna_central_id: String,
    pub lncipedia: String,
    pub gtrnadb: String,
    pub agr: String,
    pub mane_select: String,
    pub gencc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    HgncId,
    Symbol,
    Name,
    LocusGroup,
    LocusType,
    Status,
    Location,
    LocationSortable,
    AliasSymbol,
    AliasName,
    PrevSymbol,
    PrevName,
    GeneGroup,
    GeneGroupId,
    DateApprovedReserved,
    DateSymbolChanged,
    DateNameChanged,
    DateModified,
    EntrezId,
    EnsemblGeneId,
    VegaId,
    UcscId,
    Ena,
    RefseqAccession,
    CcdsId,
    UniprotIds,
    PubmedId,
    MgdId,
    RgdId,
    Lsdb,
    Cosmic,
    OmimId,
    Mirbase,
    Homeodb,
    Snornabase,
    BioparadigmsSlc,
    Orphanet,
    PseudogeneOrg,
    HordeId,
    Merops,
    Imgt,
    Iuphar,
    KznfGeneCatalog,
    MamitTrnadb,
    Cd,
    Lncrnadb,
    EnzymeId,
    IntermediateFilamentDb,
    RnaCentralId,
    Lncipedia,
    Gtrnadb,
    Agr,
    ManeSelect,
    Gencc,
}

impl Field {
    pub fn as_str(&self) -> &'static str {
        match self {
            Field::HgncId => "hgnc_id",
            Field::Symbol => "symbol",
            Field::Name => "name",
            Field::LocusGroup => "locus_group",
            Field::LocusType => "locus_type",
            Field::Status => "status",
            Field::Location => "location",
            Field::LocationSortable => "location_sortable",
            Field::AliasSymbol => "alias_symbol",
            Field::AliasName => "alias_name",
            Field::PrevSymbol => "prev_symbol",
            Field::PrevName => "prev_name",
            Field::GeneGroup => "gene_group",
            Field::GeneGroupId => "gene_group_id",
            Field::DateApprovedReserved => "date_approved_reserved",
            Field::DateSymbolChanged => "date_symbol_changed",
            Field::DateNameChanged => "date_name_changed",
            Field::DateModified => "date_modified",
            Field::EntrezId => "entrez_id",
            Field::EnsemblGeneId => "ensembl_gene_id",
            Field::VegaId => "vega_id",
            Field::UcscId => "ucsc_id",
            Field::Ena => "ena",
            Field::RefseqAccession => "refseq_accession",
            Field::CcdsId => "ccds_id",
            Field::UniprotIds => "uniprot_ids",
            Field::PubmedId => "pubmed_id",
            Field::MgdId => "mgd_id",
            Field::RgdId => "rgd_id",
            Field::Lsdb => "lsdb",
            Field::Cosmic => "cosmic",
            Field::OmimId => "omim_id",
            Field::Mirbase => "mirbase",
            Field::Homeodb => "homeodb",
            Field::Snornabase => "snornabase",
            Field::BioparadigmsSlc => "bioparadigms_slc",
            Field::Orphanet => "orphanet",
            Field::PseudogeneOrg => "pseudogene.org",
            Field::HordeId => "horde_id",
            Field::Merops => "merops",
            Field::Imgt => "imgt",
            Field::Iuphar => "iuphar",
            Field::KznfGeneCatalog => "kznf_gene_catalog",
            Field::MamitTrnadb => "mamit-trnadb",
            Field::Cd => "cd",
            Field::Lncrnadb => "lncrnadb",
            Field::EnzymeId => "enzyme_id",
            Field::IntermediateFilamentDb => "intermediate_filament_db",
            Field::RnaCentralId => "rna_central_id",
            Field::Lncipedia => "lncipedia",
            Field::Gtrnadb => "gtrnadb",
            Field::Agr => "agr",
            Field::ManeSelect => "mane_select",
            Field::Gencc => "gencc",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name.trim() {
            "hgnc_id" => Some(Field::HgncId),
            "symbol" => Some(Field::Symbol),
            "name" => Some(Field::Name),
            "locus_group" => Some(Field::LocusGroup),
            "locus_type" => Some(Field::LocusType),
            "status" => Some(Field::Status),
            "location" => Some(Field::Location),
            "location_sortable" => Some(Field::LocationSortable),
            "alias_symbol" => Some(Field::AliasSymbol),
            "alias_name" => Some(Field::AliasName),
            "prev_symbol" => Some(Field::PrevSymbol),
            "prev_name" => Some(Field::PrevName),
            "gene_group" => Some(Field::GeneGroup),
            "gene_group_id" => Some(Field::GeneGroupId),
            "date_approved_reserved" => Some(Field::DateApprovedReserved),
            "date_symbol_changed" => Some(Field::DateSymbolChanged),
            "date_name_changed" => Some(Field::DateNameChanged),
            "date_modified" => Some(Field::DateModified),
            "entrez_id" => Some(Field::EntrezId),
            "ensembl_gene_id" => Some(Field::EnsemblGeneId),
            "vega_id" => Some(Field::VegaId),
            "ucsc_id" => Some(Field::UcscId),
            "ena" => Some(Field::Ena),
            "refseq_accession" => Some(Field::RefseqAccession),
            "ccds_id" => Some(Field::CcdsId),
            "uniprot_ids" => Some(Field::UniprotIds),
            "pubmed_id" => Some(Field::PubmedId),
            "mgd_id" => Some(Field::MgdId),
            "rgd_id" => Some(Field::RgdId),
            "lsdb" => Some(Field::Lsdb),
            "cosmic" => Some(Field::Cosmic),
            "omim_id" => Some(Field::OmimId),
            "mirbase" => Some(Field::Mirbase),
            "homeodb" => Some(Field::Homeodb),
            "snornabase" => Some(Field::Snornabase),
            "bioparadigms_slc" => Some(Field::BioparadigmsSlc),
            "orphanet" => Some(Field::Orphanet),
            "pseudogene.org" => Some(Field::PseudogeneOrg),
            "horde_id" => Some(Field::HordeId),
            "merops" => Some(Field::Merops),
            "imgt" => Some(Field::Imgt),
            "iuphar" => Some(Field::Iuphar),
            "kznf_gene_catalog" => Some(Field::KznfGeneCatalog),
            "mamit-trnadb" => Some(Field::MamitTrnadb),
            "cd" => Some(Field::Cd),
            "lncrnadb" => Some(Field::Lncrnadb),
            "enzyme_id" => Some(Field::EnzymeId),
            "intermediate_filament_db" => Some(Field::IntermediateFilamentDb),
            "rna_central_id" => Some(Field::RnaCentralId),
            "lncipedia" => Some(Field::Lncipedia),
            "gtrnadb" => Some(Field::Gtrnadb),
            "agr" => Some(Field::Agr),
            "mane_select" => Some(Field::ManeSelect),
            "gencc" => Some(Field::Gencc),
            _ => None,
        }
    }
}

impl ArchivedHgncRecord {
    pub fn field_value(&self, field: Field) -> &str {
        match field {
            Field::HgncId => &self.hgnc_id,
            Field::Symbol => &self.symbol,
            Field::Name => &self.name,
            Field::LocusGroup => &self.locus_group,
            Field::LocusType => &self.locus_type,
            Field::Status => &self.status,
            Field::Location => &self.location,
            Field::LocationSortable => &self.location_sortable,
            Field::AliasSymbol => &self.alias_symbol,
            Field::AliasName => &self.alias_name,
            Field::PrevSymbol => &self.prev_symbol,
            Field::PrevName => &self.prev_name,
            Field::GeneGroup => &self.gene_group,
            Field::GeneGroupId => &self.gene_group_id,
            Field::DateApprovedReserved => &self.date_approved_reserved,
            Field::DateSymbolChanged => &self.date_symbol_changed,
            Field::DateNameChanged => &self.date_name_changed,
            Field::DateModified => &self.date_modified,
            Field::EntrezId => &self.entrez_id,
            Field::EnsemblGeneId => &self.ensembl_gene_id,
            Field::VegaId => &self.vega_id,
            Field::UcscId => &self.ucsc_id,
            Field::Ena => &self.ena,
            Field::RefseqAccession => &self.refseq_accession,
            Field::CcdsId => &self.ccds_id,
            Field::UniprotIds => &self.uniprot_ids,
            Field::PubmedId => &self.pubmed_id,
            Field::MgdId => &self.mgd_id,
            Field::RgdId => &self.rgd_id,
            Field::Lsdb => &self.lsdb,
            Field::Cosmic => &self.cosmic,
            Field::OmimId => &self.omim_id,
            Field::Mirbase => &self.mirbase,
            Field::Homeodb => &self.homeodb,
            Field::Snornabase => &self.snornabase,
            Field::BioparadigmsSlc => &self.bioparadigms_slc,
            Field::Orphanet => &self.orphanet,
            Field::PseudogeneOrg => &self.pseudogene_org,
            Field::HordeId => &self.horde_id,
            Field::Merops => &self.merops,
            Field::Imgt => &self.imgt,
            Field::Iuphar => &self.iuphar,
            Field::KznfGeneCatalog => &self.kznf_gene_catalog,
            Field::MamitTrnadb => &self.mamit_trnadb,
            Field::Cd => &self.cd,
            Field::Lncrnadb => &self.lncrnadb,
            Field::EnzymeId => &self.enzyme_id,
            Field::IntermediateFilamentDb => &self.intermediate_filament_db,
            Field::RnaCentralId => &self.rna_central_id,
            Field::Lncipedia => &self.lncipedia,
            Field::Gtrnadb => &self.gtrnadb,
            Field::Agr => &self.agr,
            Field::ManeSelect => &self.mane_select,
            Field::Gencc => &self.gencc,
        }
    }

    pub fn selected_fields<'a>(
        &'a self,
        fields: &'a [Field],
    ) -> impl Iterator<Item = (&'static str, &'a str)> + 'a {
        fields.iter().filter_map(move |&field| {
            let value = self.field_value(field);
            (!value.is_empty()).then(|| (field.as_str(), value))
        })
    }
}

pub const ALL_FIELDS: &'static [Field] = &[
    Field::HgncId,
    Field::Symbol,
    Field::Name,
    Field::LocusGroup,
    Field::LocusType,
    Field::Status,
    Field::Location,
    Field::LocationSortable,
    Field::AliasSymbol,
    Field::AliasName,
    Field::PrevSymbol,
    Field::PrevName,
    Field::GeneGroup,
    Field::GeneGroupId,
    Field::DateApprovedReserved,
    Field::DateSymbolChanged,
    Field::DateNameChanged,
    Field::DateModified,
    Field::EntrezId,
    Field::EnsemblGeneId,
    Field::VegaId,
    Field::UcscId,
    Field::Ena,
    Field::RefseqAccession,
    Field::CcdsId,
    Field::UniprotIds,
    Field::PubmedId,
    Field::MgdId,
    Field::RgdId,
    Field::Lsdb,
    Field::Cosmic,
    Field::OmimId,
    Field::Mirbase,
    Field::Homeodb,
    Field::Snornabase,
    Field::BioparadigmsSlc,
    Field::Orphanet,
    Field::PseudogeneOrg,
    Field::HordeId,
    Field::Merops,
    Field::Imgt,
    Field::Iuphar,
    Field::KznfGeneCatalog,
    Field::MamitTrnadb,
    Field::Cd,
    Field::Lncrnadb,
    Field::EnzymeId,
    Field::IntermediateFilamentDb,
    Field::RnaCentralId,
    Field::Lncipedia,
    Field::Gtrnadb,
    Field::Agr,
    Field::ManeSelect,
    Field::Gencc,
];

/// Priority levels for different types of gene identifiers/symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Archive, Serialize, Deserialize)]
#[rkyv(attr(derive(PartialEq, PartialOrd, Ord, Copy, Clone, Eq)))]
pub enum KeyPriority {
    Alias = 0,
    Previous = 1,
    Standard = 2,
    Static = 3,
}

/// Represents a match between a search key and a record
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct Match {
    pub priority: KeyPriority,
    pub record_idx: usize,
}

impl Match {
    pub fn new(priority: KeyPriority, record_idx: usize) -> Self {
        Self {
            priority,
            record_idx,
        }
    }
}

#[derive(Debug, PartialEq, Archive, Serialize, Deserialize)]
pub struct Cache {
    pub records: Vec<HgncRecord>,
    pub map: std::collections::HashMap<String, Vec<Match>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchSelection {
    HighestPriority,
    All,
}

impl ArchivedCache {
    /// Get all matches for a given key, sorted by priority (highest first)
    pub fn matches_for(&self, key: &str) -> Option<&ArchivedVec<ArchivedMatch>> {
        let normalized_key = key.trim().to_uppercase();
        self.map.get(normalized_key.as_str())
    }

    pub fn matching_records(
        &self,
        key: &str,
        selection: MatchSelection,
    ) -> impl Iterator<Item = &ArchivedHgncRecord> {
        self.matches_for(key).into_iter().flat_map(move |matches| {
            let highest = matches.first().map(|m| m.priority);

            matches
                .iter()
                .filter(move |m| match selection {
                    MatchSelection::All => true,
                    MatchSelection::HighestPriority => Some(m.priority) == highest,
                })
                .map(move |m| {
                    let idx = m.record_idx.to_native() as usize;
                    &self.records[idx]
                })
        })
    }
}
