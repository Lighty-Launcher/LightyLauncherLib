use std::path::{Path, PathBuf};
use crate::java::{find_java_binary, JavaDistribution, JavaRuntime};
use crate::minecraft::version::version::Version;
use tokio::sync::oneshot;
use crate::minecraft::version::loaders::utils::librairies::Libraries;
use crate::minecraft::version::loaders::utils::manifest::Manifest;

pub trait Launch<'a> {
    fn get_client_path(&self) -> PathBuf;
    async fn launch(&self, path: &PathBuf);
}

impl<'a> Launch<'a> for Version<'a> {
    fn get_client_path(&self) -> PathBuf {
        self.get_game_dir().join(format!("{}.jar", self.name))
    }


    async fn launch(&self, path: &PathBuf) {

        let game_directory = self.get_game_dir();
        println!("Game directory: {:?}", game_directory);

        let jre_path = path.join("jre");
        println!("JRE path: {:?}", jre_path);

        let java_version = self.get_java_from_manifest().await.unwrap();
        println!("Java version: {:?}", java_version);

        //TODO: check if java version is compatible with the current java version
        //Check if java is already installed
        //Spécifier la version de java à utiliser

        let java_distribution = JavaDistribution::Temurin;

        // Trouver java.exe (et pas javaw.exe pour avoir la console)
        let java_path = find_java_binary(&jre_path, &java_distribution, &java_version)
            .await
            .expect("Java path not found");
        println!("java path: {:?}", java_path);

        let java_runtime = JavaRuntime::new(java_path);

        //TODO: rewrite the arguments to use a generic method

        let classpath = format!(
            "{};{}",
            //get all libraries dir with recursive search
            self.get_all_libraries_dir().await.unwrap(),
            self.get_client_path().to_string_lossy()
        );

        println!("classpath: {:?}", classpath);


        let arguments = vec![
            "-Xms1024M".to_string(),
            "-Xmx2048M".to_string(),
            "-Djava.library.path=".to_owned() + &self.get_natives_dir().display().to_string(),
            "-Dfabric.development=false".to_string(),
            "-cp".to_string(),
            classpath,
            //TODO: make a generic method to get the main class from the manifest or from the version
            //self.get_main_class_from_manifest().await.unwrap(),
            //"cpw.mods.modlauncher.Launcher".to_string(),
            "net.minecraft.client.Main".to_string(),
            //"optifine.InstallerFrame".to_string(),
            "--username".to_string(),
            "Hamadi".to_string(),
            "--version".to_string(),
            self.minecraft_version.to_string(),
            "--gameDir".to_string(),
            game_directory.to_string_lossy().to_string(),
            "--assetsDir".to_string(),
            game_directory.join("assets").to_string_lossy().to_string(),
            "--assetIndex".to_string(),
            self.minecraft_version.to_string(),
            "--uuid".to_string(),
            "37fefc81-1e26-4d31-a988-74196affc99b".to_string(),
            "--accessToken".to_string(),
            "0:37fefc81-1e26-4d31-a988-74196affc99b".to_string(),
            "--userProperties".to_string(),
            "{}".to_string(),
        ];

        println!("Java arguments: {:#?}", arguments);


        match java_runtime.execute(arguments, &game_directory).await {
            Ok(mut child) => {
                let (tx, rx) = oneshot::channel::<()>();

                // Affiche les logs Java en temps réel dans le terminal
                fn print_output(_: &(), buf: &[u8]) -> anyhow::Result<()> {
                    print!("{}", String::from_utf8_lossy(buf));
                    Ok(())
                }

                if let Err(e) = java_runtime
                    .handle_io(&mut child, print_output, print_output, rx, &())
                    .await
                {
                    eprintln!("Erreur IO: {}", e);
                }

                if let Some(pid) = child.id() {
                    println!("Processus lancé avec succès, PID: {}", pid);
                } else {
                    println!("Processus lancé avec succès, PID non disponible");
                }

                // tx.send(()); // <- à utiliser si tu veux forcer l'arrêt du process plus tard
            }
            Err(e) => {
                eprintln!("Erreur lors du lancement: {}", e);
            }
        }
    }
}
