//! Analysts Algorithm
//! 
//! The analysts algorithms are stored in a list 
//! which holds the operations of [`Atomic`] type
//! that build up the KPIs that are computed.
//! The formulas are representable as multiple 
//! trees that are directed acyclic graphs (DAGs).
//! 
//! This implementation parses algorithms and checks
//! that they are indeed acyclic.
//! The algorithm can check for possible inputs that
//! they correctly provide necessary information
//! to perform computation.
//! 
//! `run` performs the computation of [`Company`] 
//! input data and returns all KPI variables.

pub mod atomic;
pub mod variable;
pub mod operation;
pub mod resolved;
pub mod error;

use serde::{Deserialize, Serialize};
use strum::Display;
use types::input::Input;
use types::output::{OutputVariable, Output};
use variable::Variable;
use std::collections::HashSet;
use std::{fs::OpenOptions, collections::HashMap};
use std::io::BufReader;
use std::path::PathBuf;

use self::{
    atomic::Atomic,
    resolved::ResolvedValues, error::BenchmarkingError
};

use types::entity::Company;

/// Structure holds all atomic operations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Algorithm {
    /// list off all atomic formulas
    operations: Vec<Atomic>,
    #[serde(default)]
    required: HashSet<String>,

    // Optimization
    #[serde(default)]
    algohelper: AlgoHelper,
}

/// This helper is only for initialization
/// After this phase there is no consistency
/// guaraneed!
#[derive(Serialize, Deserialize, Clone, Debug)]
struct AlgoHelper {
    // Lookup in O(1) → Atomics and other
    op_lookup: HashMap<String, Atomic>,
    kpis: Vec<Atomic>,
    non_kpis: HashSet<String>,
}


impl Default for AlgoHelper {
    fn default() -> Self {
        AlgoHelper { op_lookup: HashMap::new(), kpis: Vec::new(), non_kpis: HashSet::new()}
    }
}

#[derive(Clone, PartialEq, Display)]
enum DepState {
    Unresolved,
    InVisit,
    Resolved,
}

impl Algorithm {
    /// Pareses the algorithm file
    pub fn load(path: &PathBuf) -> Result<(Self, u128, u128), BenchmarkingError> {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .expect("Could not open ATOMIC_FILE");

        #[cfg(feature="evaluation")]
        let now = std::time::SystemTime::now();

        let buffer = BufReader::new(file);
        let mut res: Self = serde_yaml::from_reader(buffer).map_err(|err| BenchmarkingError::from(format!("Could not parse algorithm: {}", err.to_string())))?;

        #[cfg(feature="evaluation")]
        let parse_time = now.elapsed().unwrap().as_nanos();
        
        #[cfg(not(feature="evaluation"))]
        {
            // Sanity check: is not empty and operations are unique
            let mut unique_elems: std::collections::HashSet<String> = std::collections::HashSet::new();
            let has_unique_elems = res.operations.iter().all(move |x| unique_elems.insert(x.name().to_string()));

            if res.len() == 0 ||  !has_unique_elems {
                return Err(
                    BenchmarkingError::from(format!("The provided algorihm is malformed, because {}{}", 
                    if !has_unique_elems {"it has duplicate definitions"} else {""}, 
                    if res.len() == 0 {"it is empty"} else {""}))
                );
            }
        }

        #[cfg(feature="evaluation")]
        let now2 = std::time::SystemTime::now();

        // Create lookup tables and results
        for i in &res.operations {
            let atom = i.clone();
            
            if i.is_kpi() { res.algohelper.kpis.push(atom.clone());} 
            else { res.algohelper.non_kpis.insert(atom.name().to_string());}

            res.algohelper.op_lookup.insert(atom.name().to_string(), atom);
        }

        res.required = HashSet::with_capacity(res.operations().len());

        // Extend dependency graph to constants
        for op in res.operations.clone() {
            log::debug!("{:?}", op);
            for subop in op.var_ids() {
                log::debug!("{} - {}", subop, ! res.has_atomic_as_var(&subop));
                if ! res.has_atomic_as_var(&subop) {

                    let required_input_atom = Atomic::new_required(subop.clone());
                    
                    // Lookup tables and extension
                    res.required.insert(required_input_atom.name().to_string());
                    res.operations.push(required_input_atom.clone());
                    res.algohelper.op_lookup.insert(required_input_atom.name().to_string(), required_input_atom.clone());
                    res.algohelper.non_kpis.insert(required_input_atom.name().to_string());

                }
            }    
        }

        // Overwrite operations: Now they are ordered by topological execution
        res.operations = res.topological_op_sort()?;
        
        #[cfg(feature="evaluation")]
        {
            let topo_time =  now2.elapsed().unwrap().as_nanos();
            Ok((res, parse_time, topo_time))
        }
        



        #[cfg(not(feature="evaluation"))]
        Ok((res, 0, 0))
    }

    /// Resolve dependencies and verify the computability of the algorithm
    /// This yields a vector which defines the necessary order for computation  
    /// This provides the order in which the algorithm has to be computed
    /// and further provides the necessary dependencies for companies, as 
    /// each atomic with AdditionConst (0) is surely required as an input.
    pub fn topological_op_sort(&self) -> Result<Vec<Atomic>, BenchmarkingError> {

        // Create the map to do ordering
        let mut topo: HashMap<&str, (usize, DepState)> = HashMap::with_capacity(self.len());
        for i in &self.operations {
            topo.insert(i.name(), (0, DepState::Unresolved));
        }
    
        // Topological ordering
        let mut topo_num: usize = 0;
        self.dfs_topo_sort(&mut topo, &mut topo_num)?;

        // Collect information from result map
        let mut resolution_ordering: Vec<(usize, Atomic)> = Vec::new();
        let map_vec: Vec<(&&str, &(usize, DepState))> = topo.iter().collect();
        for i in map_vec {
            let atom = self.find_atomic_by_name(i.0).unwrap().clone();
            
            // Unrequired input variables variables will not be
            // coputed since they exist already.
            if ! self.required.contains(atom.name()) {
                resolution_ordering.push((i.1.0, atom.clone()));
            }
        }

        log::debug!("ORDER: {:?}", resolution_ordering.clone());

        // Sort results and return the operations in topological ordering
        resolution_ordering.sort_by(|a,b|a.0.cmp(&b.0));
        Ok(resolution_ordering.iter().map(|a| a.1.clone()).collect())
    }

    /// Caller function for all KPIs in case of a dependency graph consisting of multiple trees
    fn dfs_topo_sort(&self, topo: &mut HashMap<&str, (usize, DepState)>, topo_num: &mut usize) -> Result<(), BenchmarkingError> {
        
        // TODO: Parallelization
        
        for i in self.operations.clone() {
            if topo.get(i.name()).unwrap().1 == DepState::Unresolved {
                self.dfs_topo_sort_inner(topo, i.name(), topo_num)?;
            }
        }
        Ok(())
    }

    /// Handle topological sorting on one tree
    fn dfs_topo_sort_inner(&self, topo: &mut HashMap<&str, (usize, DepState)>, curr_op: &str, topo_num: &mut usize) -> Result<(), BenchmarkingError> {
    
        let tuple = topo.get_mut(curr_op).unwrap();
        tuple.1 = DepState::InVisit;
        log::debug!("children: {:?}", self.find_atomic_by_name(curr_op).unwrap().var_ids());

        for i in self.find_atomic_by_name(curr_op).unwrap().var_ids() {
            log::debug!("{}", i.clone());
            match topo.get(i.as_str()).unwrap().1 {
                DepState::InVisit => { return Err(BenchmarkingError::from("Cyclic Dependency Hell detected - what kind of an analyst are you?".to_string())); }, // Cyclic dependency
                DepState::Unresolved => { self.dfs_topo_sort_inner(topo, &i, topo_num)?; }, // Go one step deeper
                DepState::Resolved => {}, // Ok, dependency served
            }

        }

        // Update the toponumber
        topo.get_mut(curr_op).unwrap().0 = *topo_num;
        topo.get_mut(curr_op).unwrap().1 = DepState::Resolved;
        *topo_num += 1;
        
        Ok(())
    }

    /// Returns the number of operations
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// return all operations of `self`
    pub fn operations(&self) -> &Vec<Atomic> {
        &self.operations
    }

    // Required operations are of form AdditionConst 0 (Invariant of dfs_topo_sort!)
    fn required_input_atomics(&self) -> &HashSet<String> {
        &self.required
    }

    // Verifies the input of a company and throws error in case of missing fields
    pub fn verify_input(&self, input: Input) -> Result<(), BenchmarkingError> {
        
        let required: &HashSet<String> = self.required_input_atomics();
        let missing_vars: Vec<&String> = required.iter().filter(|&req_atom| !input.has_input_var(req_atom)).collect();

        // At least one variable is missing
        if missing_vars.len() != 0 {
            log::error!("Missing: {:?}", missing_vars);
            Err(BenchmarkingError::from(format!("Missing input variables: {:?}", missing_vars)))
        }else{
            Ok(())
        }
    }

    /// return all non-kpis
    pub fn get_non_kpis(&self) -> &HashSet<String> {
        &self.algohelper.non_kpis
    }

    /// return all kpis
    pub fn get_kpis(&self) -> &Vec<Atomic> {
        &self.algohelper.kpis
    }

    /// Given a name, this returns whether an atomic with this name exists and returns a reference to it 
    fn find_atomic_by_name(&self, id: &str) -> Option<&Atomic> {
        self.algohelper.op_lookup.get(id)
    }

    /// Given an name return whether an atomic with this name exists
    fn has_atomic_as_var(&self, id: &str) -> bool {
        self.algohelper.op_lookup.contains_key(id)
    }

    /// Run the algorithm with company input data
    /// 
    /// This calls the opertions (that were previously topologically sorted)
    /// in sequential order and computes all formula values. Especially are
    /// "resolved values" computed, that are not KPIs. They get removed in 
    /// the end as they are not required as results. 
    pub fn run(&self, company: &Company) -> Result<(Output, u128), BenchmarkingError> {

        let mut resolved_vals = ResolvedValues::new();

        // Store all "initially" resolved vars
        let req = self.required_input_atomics();
        for i in req {
            // Since we verified before unwrapping is safe
            let var = Variable::new(company.input_data().get_input_var(&i).unwrap().values().to_vec());
            log::debug!("var: {:?} with val {:?}", i, var);
            resolved_vals.insert(i.to_string(), var)?;
        }

        #[cfg(feature="evaluation")]
        let mut ops_time: u128 = 0;

        // Compute every atomic in given order
        //
        // Computation will NOT fail because of missing 
        // values since we verified them in advance.
        // Still there might be runtime errors e.g.
        // unused constants.. that get reported! 
        for atom in self.operations.iter() {
            log::info!("Computing op: {}", atom.name());

            #[cfg(feature="evaluation")]
            let now = std::time::SystemTime::now();

            atom.calc(&mut resolved_vals)?;

            #[cfg(feature="evaluation")]
            {
                ops_time +=  now.elapsed().unwrap().as_nanos();
            }
            
        }

        // Return only relevant data - rest is discarded
        resolved_vals.filter_atomics_by_name(self.get_non_kpis());
        resolved_vals.filter_atomics_by_name(&self.required);

        #[cfg(not(feature="evaluation"))]
        {
            // Debug: print the values
            resolved_vals.print_plain();
        }

        // Add output variables and append them to the company
        let mut output_vars = Output::new_empty();
        for i in resolved_vals.resolved() {
            let out_var = OutputVariable::new_result_only(i.0.clone(), (*i.1.vector()).to_vec());
            output_vars.add_var(out_var);
        }
        

        #[cfg(feature="evaluation")]
        {
            Ok((output_vars, ops_time / self.operations.len() as u128))
        }
        
            
        #[cfg(not(feature="evaluation"))]
        Ok((output_vars, 0))
    }

    /// Aggregate one variable which was evaluated among many companies
    /// 
    /// This yields the min, max, lq, median, uq and average of one KPI 
    /// from all company results, that were computed.
    pub fn aggregate_atomic_var(&self, company_results: &mut Vec<&Vec<f64>>) -> Result<OutputVariable, BenchmarkingError> {

        log::info!("Aggregation on KPI with input {:?} started", company_results);

        #[cfg(not(feature="evaluation"))]
        {
            // sanity check: no nans and infs
            for company_result in company_results.iter() {
                for entry in company_result.iter(){
                    if (*entry).is_infinite(){
                        log::error!("Invalid computation result - return empty results!");
                        return Ok(OutputVariable::new("".to_string(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()))
                    }
                }
            }    
        }
        
        // Statistical measurements that rely on ranks
        company_results.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        
        let median = if company_results.len() % 2 == 0 {
            let s0 = company_results[(company_results.len() / 2) as usize - 1 ].clone();
            let s1 = company_results[(company_results.len() / 2) as usize     ].clone();
            let both = s0.iter().zip(s1);
            both.map(|r| (r.0 + r.1) / 2_f64).collect()
        }else {
            company_results[(company_results.len() / 2) as usize].clone()
        };
        
        let lq = if company_results.len() % 4 == 0 {
            let s0 = company_results[(company_results.len() / 4) as usize - 1].clone();
            let s1 = company_results[(company_results.len() / 4) as usize    ].clone();
            let both = s0.iter().zip(s1);
            both.map(|r| (r.0 + r.1) / 2_f64).collect()
        }else {
            company_results[(company_results.len() / 4) as usize].clone()
        };
        let uq = if company_results.len() % 4 == 0 {
            let s0 = company_results[(3 * company_results.len() / 4) as usize -1].clone();
            let s1 = company_results[(3 * company_results.len() / 4) as usize   ].clone();
            let both = s0.iter().zip(s1);
            both.map(|r| (r.0 + r.1) / 2_f64).collect()
        }else {
            company_results[(3 * company_results.len() / 4) as usize].clone()
        };
        
        let min = company_results.first().unwrap().clone();
        let max = company_results.last().unwrap().clone();

    
        // Average
        // Component wise addition of values where they are divided by the number of participants
        // We assume all participants to have the same amount of entries → use 0 as reference

        let kpi_dimension = company_results[0].len();
        let mut result_kpi_sum: Vec<f64> = Vec::new();
        
        // Dimension wise addition
        for i in 0..kpi_dimension {
            let mut component_sum: f64 = 0_f64;
            
            for company_result in company_results.iter() {
                component_sum += company_result[i];
            }

            result_kpi_sum.push(component_sum);
        }
        let avg: Vec<f64> = result_kpi_sum.iter().map(|sum_acc| sum_acc / company_results.len() as f64).collect();

        let output = OutputVariable::new("".to_string(), Vec::new(), max.to_vec(), min.to_vec(), avg, median, lq, uq);

        Ok(output)
    }
}
