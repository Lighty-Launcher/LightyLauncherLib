#![feature(duration_constructors)]
extern crate core;
mod utils;
mod java;
mod minecraft;
#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use directories::ProjectDirs;
    use std::fmt::{Debug, Display};
    use crate::minecraft::version::launch::Launch;
    use super::minecraft::version::version;

    #[tokio::test]
    async fn test_os() {
        // Create the logs directory if is not blocked


        pub const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");
        static LAUNCHER_DIRECTORY: Lazy<ProjectDirs> =
            Lazy::new(
                || match ProjectDirs::from("fr", ".LightyLauncher","") {
                    Some(proj_dirs) => proj_dirs,
                    None => panic!("no application directory"),
                },
            );
        let logs = LAUNCHER_DIRECTORY.data_dir().join("logs");



        let minozia = version::Version::new("minozia", "fabric", "0.15.10", "1.20.2",&LAUNCHER_DIRECTORY);
        let wynlers = version::Version::new("wynlers", "vanilla", "", "1.7.10",&LAUNCHER_DIRECTORY);
        let cobblemonfr = version::Version::new("cobblemonfr", "vanilla", "", "1.21.5",&LAUNCHER_DIRECTORY);
        let nemaria = version::Version::new("nemaria", "vanilla", "", "1.20.2",&LAUNCHER_DIRECTORY);
        let frozenearth = version::Version::new("frozenearth", "fabric", "0.15.10", "1.14",&LAUNCHER_DIRECTORY);
        let ephesia = version::Version::new("ephesia", "quilt", "0.17.10", "1.18.2",&LAUNCHER_DIRECTORY);
        let sodacraft = version::Version::new("sodacraft", "optifine", "", "1.18.2",&LAUNCHER_DIRECTORY);
        let gaïa = version::Version::new("gaïa", "neoforge", "47.1.99", "1.20.1",&LAUNCHER_DIRECTORY);


        //minozia.install_version().await.unwrap();
        //minozia.launch(&LAUNCHER_DIRECTORY.config_dir().to_path_buf()).await;
        //frozenearth.install_version().await.unwrap();
        //frozenearth.launch(&LAUNCHER_DIRECTORY.config_dir().to_path_buf()).await;
        gaïa.install_version().await.unwrap();

        //TODO: déplacer le config_dir dans la méthode launch
        //gaïa.launch(&LAUNCHER_DIRECTORY.config_dir().to_path_buf()).await;

        // ephesia.install_version().await.unwrap();
        // ephesia.launch(&LAUNCHER_DIRECTORY.config_dir().to_path_buf()).await;
        // cobblemonfr.install_version().await.unwrap();
        // nemaria.install_version().await.unwrap();
        // wynlers.install_version().await.unwrap();
        //wynlers.launch(&LAUNCHER_DIRECTORY.config_dir().to_path_buf()).await;
        //nemaria.launch(&LAUNCHER_DIRECTORY.config_dir().to_path_buf()).await;
        //cobblemonfr.launch(&LAUNCHER_DIRECTORY.config_dir().to_path_buf()).await;






        //println!("wynlers all libraries {:#?}", wynlers.get_all_libraries_dir().await);

         //wynlers.uninstall_version().await.unwrap();
        // wynlers.install_vanilla_version().await.unwrap();

        // frozenearth.install_version().await;


        // // Minozia test directory
        // println!("minozia path {:?}",minozia.get_game_dir());
        // mkdir!(minozia.get_game_dir());
        // println!("minozia {:?}",minozia);
        // minozia.install_version().await;
        //
        // // Wynlers test directory
        // println!("wynlers {:?}",wynlers);
        // wynlers.install_version().await;
        // println!("wynlers path {:?}",wynlers.get_game_dir());
        // mkdir!(wynlers.get_game_dir());


        // cobblemonfr.ensure_game_dir_exists().unwrap()


    }
}

// //Clean the logs directory
// if let Err(e) = utils::system::clean_directory(&logs, 7) {
//     error!("Failed to clear log folder: {:?}", e);
// }
//
// {
//     let span = debug_span!("startup");
//     let _guard = span.enter();
//
//     // Set up the logger
//     info!(parent: &span, "Starting LightyLauncher v{}", LAUNCHER_VERSION);
//     info!(parent: &span, "OS: {:} {:} {:}", OS, ARCHITECTURE, OS_VERSION.to_string());
//     // application directory
//     info!(parent: &span, "Creating application directory");
//     debug!(parent: &span, "Application directory: {:?}", LAUNCHER_DIRECTORY.data_dir());
//     debug!(parent: &span, "Config directory: {:?}", LAUNCHER_DIRECTORY.config_dir());
//     debug!(parent: &span, "JRE directory: {:?}", LAUNCHER_DIRECTORY.config_dir().join("jre"));
//     mkdir!(LAUNCHER_DIRECTORY.data_dir());
//     mkdir!(LAUNCHER_DIRECTORY.config_dir());
//     mkdir!(LAUNCHER_DIRECTORY.config_dir().join("jre"));
//
//  }


// let jre_path = LAUNCHER_DIRECTORY.config_dir().join("jre");
// let runtime = tokio::runtime::Runtime::new().unwrap();
// let jre = runtime.block_on(jre_download(
//     &jre_path,
//     &JavaDistribution::Temurin,
//     &11u32,
//     |current, total| {
//         // called function to update the progress
//         println!("Téléchargement : {}/{}", current, total);
//     }
// ));
//
// let find_jre_8 = runtime.block_on(find_java_binary(
//     &jre_path,
//     &JavaDistribution::Temurin,
//     &8u32));
// println!("jre {:?}",&find_jre_8);
// let find_jre_11 = runtime.block_on(find_java_binary(
//     &jre_path,
//     &JavaDistribution::Temurin,
//     &11u32));
// println!("jre {:?}",&find_jre_11);
// let find_jre_17 = runtime.block_on(find_java_binary(
//     &jre_path,
//     &JavaDistribution::Temurin,
//     &17u32));
// println!("jre {:?}",&find_jre_17);
// let find_jre_21 = runtime.block_on(find_java_binary(
//     &jre_path,
//     &JavaDistribution::Temurin,
//     &21u32));
// println!("jre {:?}",&find_jre_21);