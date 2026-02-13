use rustc_hash::{FxBuildHasher, FxHashMap};

const K: usize = 21;
const MIN_VALID_DEPTH: usize = 10; //huristic so just don't let it be affected by low depth reads (ONT HIGH ERROR)

#[derive(Debug)]
pub struct FastqStats {
    total_bases: usize,
    total_reads: usize,
    length_counts: FxHashMap<usize, usize>,
    n50: Option<usize>,
    kmer_counts: FxHashMap<u64, usize>,
    genome_size: Option<usize>,
}

impl FastqStats {
    pub fn new() -> Self {
        Self {
            total_bases: 0,
            total_reads: 0,
            length_counts: FxHashMap::default(),
            kmer_counts: FxHashMap::with_capacity_and_hasher(10_000_000, FxBuildHasher::default()),
            n50: None,
            genome_size: None,
        }
    }

    pub fn add_read(&mut self, sequence: &[u8], genome_size: bool) {
        let len = sequence.len();
        self.total_reads += 1;
        self.total_bases += len;
        *self.length_counts.entry(len).or_insert(0) += 1;

        if genome_size && self.total_reads % 5 == 0 {
            self.hash_kmers(sequence);
        }
    }

    pub fn calculate_n50(&mut self) -> usize {
        if let Some(n50) = self.n50 {
            return n50;
        }

        let mut lengths: Vec<(usize, usize)> = self
            .length_counts
            .iter()
            .map(|(&len, &count)| (len, count))
            .collect();
        lengths.sort_unstable_by(|a, b| b.0.cmp(&a.0));

        let half_bases = self.total_bases / 2;
        let mut cumulative = 0;

        for (len, count) in lengths {
            cumulative += len * count;
            if cumulative >= half_bases {
                self.n50 = Some(len);
                return len;
            }
        }

        0
    }

    #[inline]
    fn to_bits(b: u8) -> Option<u64> {
        match b {
            b'A' | b'a' => Some(0),
            b'C' | b'c' => Some(1),
            b'G' | b'g' => Some(2),
            b'T' | b't' => Some(3),
            _ => None, //N
        }
    }

    fn hash_kmers(&mut self, sequence: &[u8]) {
        let mut forward: u64 = 0;
        let mut reverse: u64 = 0;

        let mask: u64 = (1u64 << (2 * K)) - 1;
        let mut valid_bases = 0;

        for &base in sequence {
            match Self::to_bits(base) {
                Some(bits) => {
                    forward = ((forward << 2) | bits) & mask;

                    let comp = bits ^ 0b11;
                    reverse = (reverse >> 2) | (comp << (2 * (K - 1)));

                    valid_bases += 1;

                    if valid_bases >= K {
                        let canonical = forward.min(reverse);
                        *self.kmer_counts.entry(canonical).or_insert(0) += 1;
                    }
                }
                None => {
                    forward = 0;
                    reverse = 0;
                    valid_bases = 0;
                }
            }
        }
    }

    pub fn calculate_genome_size(&mut self) {
        // Remove likely sequencing errors
        self.kmer_counts.retain(|_, v| *v > 5);

        let mut depth_hist: FxHashMap<usize, usize> = FxHashMap::default();

        for &count in self.kmer_counts.values() {
            if count > 1 {
                *depth_hist.entry(count).or_insert(0) += 1;
            }
        }

        let peak_depth = depth_hist
            .iter()
            .filter(|&(&depth, _)| depth >= MIN_VALID_DEPTH)
            .max_by_key(|(_, freq)| *freq)
            .map(|(depth, _)| *depth)
            .unwrap();

        let total_kmers: usize = self.kmer_counts.values().sum();
        self.genome_size = Some(total_kmers / peak_depth);
    }

    pub fn to_yaml(&self) -> String {
        let mut yaml = String::new();

        yaml.push_str("FastqStats:\n");
        yaml.push_str(&format!("  total_reads: {}\n", self.total_reads));
        yaml.push_str(&format!("  total_bases: {}\n", self.total_bases));

        if let Some(n50) = self.n50 {
            yaml.push_str(&format!("  n50: {}\n", n50));
        } else {
            yaml.push_str("  n50: null\n");
        }

        if let Some(genome_size) = self.genome_size {
            yaml.push_str(&format!("  genome_size: {}\n", genome_size));
        } else {
            yaml.push_str("  genome_size: null\n");
        }

        yaml.push_str(&format!("  unique_kmers: {}\n", self.kmer_counts.len()));
        yaml.push_str(&format!(
            "  unique_read_lengths: {}\n",
            self.length_counts.len()
        ));

        yaml
    }
}
