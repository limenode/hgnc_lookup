use rkyv::{Archive, Deserialize, Serialize, vec::ArchivedVec};

#[derive(serde::Deserialize, Debug, PartialEq, Archive, Serialize, Deserialize)]
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

impl ArchivedHgncRecord {
    pub fn get_field(&self, name: &str) -> Option<&str> {
        match name {
            "hgnc_id" => Some(&self.hgnc_id),
            "symbol" => Some(&self.symbol),
            "name" => Some(&self.name),
            "locus_group" => Some(&self.locus_group),
            "locus_type" => Some(&self.locus_type),
            "status" => Some(&self.status),
            "location" => Some(&self.location),
            "location_sortable" => Some(&self.location_sortable),
            "alias_symbol" => Some(&self.alias_symbol),
            "alias_name" => Some(&self.alias_name),
            "prev_symbol" => Some(&self.prev_symbol),
            "prev_name" => Some(&self.prev_name),
            "gene_group" => Some(&self.gene_group),
            "gene_group_id" => Some(&self.gene_group_id),
            "date_approved_reserved" => Some(&self.date_approved_reserved),
            "date_symbol_changed" => Some(&self.date_symbol_changed),
            "date_name_changed" => Some(&self.date_name_changed),
            "date_modified" => Some(&self.date_modified),
            "entrez_id" => Some(&self.entrez_id),
            "ensembl_gene_id" => Some(&self.ensembl_gene_id),
            "vega_id" => Some(&self.vega_id),
            "ucsc_id" => Some(&self.ucsc_id),
            "ena" => Some(&self.ena),
            "refseq_accession" => Some(&self.refseq_accession),
            "ccds_id" => Some(&self.ccds_id),
            "uniprot_ids" => Some(&self.uniprot_ids),
            "pubmed_id" => Some(&self.pubmed_id),
            "mgd_id" => Some(&self.mgd_id),
            "rgd_id" => Some(&self.rgd_id),
            "lsdb" => Some(&self.lsdb),
            "cosmic" => Some(&self.cosmic),
            "omim_id" => Some(&self.omim_id),
            "mirbase" => Some(&self.mirbase),
            "homeodb" => Some(&self.homeodb),
            "snornabase" => Some(&self.snornabase),
            "bioparadigms_slc" => Some(&self.bioparadigms_slc),
            "orphanet" => Some(&self.orphanet),
            "pseudogene.org" => Some(&self.pseudogene_org),
            "horde_id" => Some(&self.horde_id),
            "merops" => Some(&self.merops),
            "imgt" => Some(&self.imgt),
            "iuphar" => Some(&self.iuphar),
            "kznf_gene_catalog" => Some(&self.kznf_gene_catalog),
            "mamit-trnadb" => Some(&self.mamit_trnadb),
            "cd" => Some(&self.cd),
            "lncrnadb" => Some(&self.lncrnadb),
            "enzyme_id" => Some(&self.enzyme_id),
            "intermediate_filament_db" => Some(&self.intermediate_filament_db),
            "rna_central_id" => Some(&self.rna_central_id),
            "lncipedia" => Some(&self.lncipedia),
            "gtrnadb" => Some(&self.gtrnadb),
            "agr" => Some(&self.agr),
            "mane_select" => Some(&self.mane_select),
            "gencc" => Some(&self.gencc),
            _ => None,
        }
    }
}

pub const ALL_FIELDS: &'static [&'static str] = &[
    "hgnc_id",
    "symbol",
    "name",
    "locus_group",
    "locus_type",
    "status",
    "location",
    "location_sortable",
    "alias_symbol",
    "alias_name",
    "prev_symbol",
    "prev_name",
    "gene_group",
    "gene_group_id",
    "date_approved_reserved",
    "date_symbol_changed",
    "date_name_changed",
    "date_modified",
    "entrez_id",
    "ensembl_gene_id",
    "vega_id",
    "ucsc_id",
    "ena",
    "refseq_accession",
    "ccds_id",
    "uniprot_ids",
    "pubmed_id",
    "mgd_id",
    "rgd_id",
    "lsdb",
    "cosmic",
    "omim_id",
    "mirbase",
    "homeodb",
    "snornabase",
    "bioparadigms_slc",
    "orphanet",
    "pseudogene.org",
    "horde_id",
    "merops",
    "imgt",
    "iuphar",
    "kznf_gene_catalog",
    "mamit-trnadb",
    "cd",
    "lncrnadb",
    "enzyme_id",
    "intermediate_filament_db",
    "rna_central_id",
    "lncipedia",
    "gtrnadb",
    "agr",
    "mane_select",
    "gencc",
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

impl ArchivedCache {
    /// Get all matches for a given key, sorted by priority (highest first)
    pub fn get_matches(&self, key: &str) -> Option<&ArchivedVec<ArchivedMatch>> {
        let normalized_key = key.trim().to_uppercase();

        self.map.get(normalized_key.as_str())
    }

    /// Get indices based on whether to return all matches or just highest priority
    /// Get indices of matching records, optionally filtered by highest priority
    pub fn get_indices(&self, key: &str, return_all: bool) -> Option<Vec<usize>> {
        let matches = self.get_matches(key)?;

        if matches.is_empty() {
            return None;
        }

        if return_all {
            // Return all match indices
            Some(
                matches
                    .iter()
                    .map(|m| m.record_idx.to_native() as usize)
                    .collect(),
            )
        } else {
            // matches are sorted by priority, first has highest priority
            let highest_priority = matches[0].priority;

            // Take while priority matches
            Some(
                matches
                    .iter()
                    .take_while(|m| m.priority == highest_priority)
                    .map(|m| m.record_idx.to_native() as usize)
                    .collect(),
            )
        }
    }
}
