use crate::imports::*;

#[derive(Default, Handler)]
#[help("Import a wallet, mnemonic, or a private key")]
pub struct Import;

impl Import {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, argv: Vec<String>, _cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<KaspaCli>()?;
        let wallet = ctx.wallet();

        if argv.is_empty() {
            self.display_help(ctx).await?;
            return Ok(());
        }

        let what = argv.get(0).unwrap();
        match what.as_str() {
            "mnemonic" => {
                crate::wizards::import::import_with_mnemonic(&ctx).await?;
            }
            "legacy" => {
                if exists_legacy_v0_keydata().await? {
                    let import_secret = Secret::new(
                        ctx.term().ask(true, "Enter the password for the wallet you are importing:").await?.trim().as_bytes().to_vec(),
                    );
                    let wallet_secret = Secret::new(ctx.term().ask(true, "Enter wallet password:").await?.trim().as_bytes().to_vec());
                    wallet.import_gen0_keydata(import_secret, wallet_secret).await?;
                } else if application_runtime::is_web() {
                    return Err("'kaspanet' web wallet storage not found at this domain name".into());
                } else {
                    return Err("KDX/kaspanet keydata file not found".into());
                }
            }
            // "core" => {}
            v => {
                tprintln!(ctx, "unknown command: '{v}'\r\n");
                return self.display_help(ctx).await;
            }
        }

        Ok(())
    }

    async fn display_help(self: Arc<Self>, ctx: Arc<KaspaCli>) -> Result<()> {
        ctx.term().help(
            &[
                ("mnemonic", "Import a 24 or 12 word mnemonic"),
                ("legacy", "Import a legacy (local KDX) wallet"),
                // ("purge", "Purge an account from the wallet"),
            ],
            None,
        )?;

        Ok(())
    }
}
