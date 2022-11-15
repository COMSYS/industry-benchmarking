pub mod analyst;
pub mod company;
pub mod spectator;
pub mod state;

pub mod connection {
    use super::{analyst::AnalystConnection, state::ClientConnection, company::CompanyConnection, spectator::SpectatorConnection};

    pub enum TeebenchClient {
        Analyst(AnalystConnection),
        Company(CompanyConnection),
        Spectator(SpectatorConnection),
    }

    impl TeebenchClient {
        pub async fn run(&mut self) {
            match self {
                Self::Analyst(conn) => {
                    conn.run().await;
                },
                Self::Company(conn) => {
                    conn.run().await;
                },
                Self::Spectator(conn) => {
                    conn.run().await;
                }
            }
        }
    }
}