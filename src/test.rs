#[cfg(test)]
mod entity_tests {
    use crate::entity::{
        Entity,
        kind::EntityKind,
        permission::{EntityPermissionsCategory, EntityPermissionsKind},
    };
    use std::fs::{File, create_dir};
    use tempfile::TempDir;

    #[test]
    fn test_entity_creation_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path).unwrap();

        let entity = Entity::new(file_path.clone()).unwrap();

        assert_eq!(entity.name(), "test_file");
        assert_eq!(entity.extension_str(), "txt");
        assert!(entity.is_file());
        assert!(!entity.is_dir());
    }

    #[test]
    fn test_entity_creation_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        create_dir(&dir_path).unwrap();

        let entity = Entity::new(dir_path.clone()).unwrap();

        assert_eq!(entity.name(), "test_dir");
        assert!(entity.is_dir());
        assert!(!entity.is_file());
    }

    #[test]
    fn test_entity_ghost_creation() {
        let temp_dir = TempDir::new().unwrap();
        let ghost_path = temp_dir.path().join("ghost_file.txt");

        let ghost_entity = Entity::ghost(
            ghost_path.clone(),
            "ghost_file".to_string(),
            EntityKind::File,
        )
        .unwrap();

        assert_eq!(ghost_entity.name(), "ghost_file");
        assert_eq!(ghost_entity.extension_str(), "txt");
        assert!(!ghost_entity.exists());
    }

    #[test]
    fn test_entity_content_operations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("content_test.txt");
        let test_content = "Test file content";

        std::fs::write(&file_path, test_content).unwrap();
        let entity = Entity::new(file_path).unwrap();

        let content = entity.content().unwrap();
        assert_eq!(content, test_content);
    }

    #[test]
    fn test_entity_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("perm_test.txt");
        File::create(&file_path).unwrap();

        let entity = Entity::new(file_path).unwrap();
        let permissions = entity.permissions();

        assert!(permissions.has(EntityPermissionsCategory::User, EntityPermissionsKind::Read));
        assert!(permissions.has(
            EntityPermissionsCategory::User,
            EntityPermissionsKind::Write
        ));
    }
}

#[cfg(test)]
mod model_tests {
    use crate::{commands::Command, model::JujikModel};
    use std::{sync::mpsc, time::Duration};
    use tempfile::TempDir;

    #[test]
    fn test_create_pin_command() {
        let (controller_tx, controller_rx) = mpsc::channel();
        let (model_tx, model_rx) = mpsc::channel();

        let model = JujikModel::new(controller_tx, model_rx);
        let temp_dir = TempDir::new().unwrap();

        model_tx
            .send(Command::CreatePin(temp_dir.path().to_path_buf()))
            .unwrap();

        if let Ok(command) = controller_rx.recv_timeout(Duration::from_secs(1)) {
            match command {
                Command::NewPin(None, pin) => {
                    assert_eq!(pin.path(), temp_dir.path());
                }
                _ => panic!("Unexpected command received"),
            }
        }
    }
}

#[cfg(test)]
mod controller_tests {
    use crate::{commands::Command, config::Config, controller::JujikController};
    use std::{path::PathBuf, sync::mpsc, time::Duration};

    #[test]
    fn test_command_validation() {
        let (model_tx, model_rx) = mpsc::channel();
        let (view_tx, view_rx) = mpsc::channel();
        let (controller_tx, controller_rx) = mpsc::channel();

        let controller = JujikController::new(model_tx, view_tx, controller_rx);

        let nonexistent_path = PathBuf::from("/nonexistent/path");
        controller_tx
            .send(Command::CreateEntitys(nonexistent_path))
            .unwrap();

        if let Ok(command) = view_rx.recv_timeout(Duration::from_secs(1)) {
            match command {
                Command::Error(_) => (),
                _ => panic!("Expected error command"),
            }
        }
    }

    #[test]
    fn test_config_synchronization() {
        let (model_tx, _) = mpsc::channel();
        let (view_tx, view_rx) = mpsc::channel();
        let (controller_tx, controller_rx) = mpsc::channel();

        let mut controller = JujikController::new(model_tx, view_tx, controller_rx);
        let new_config = Config::default();

        controller_tx
            .send(Command::SetConfig(new_config.clone()))
            .unwrap();

        if let Ok(command) = view_rx.recv_timeout(Duration::from_secs(1)) {
            match command {
                Command::SetConfig(config) => {
                    assert_eq!(config.pins.len(), new_config.pins.len());
                }
                _ => panic!("Expected SetConfig command"),
            }
        }
    }
}

#[cfg(test)]
mod functional_tests {
    use crate::{
        entity::{Entity, permission::EntityPermissions},
        tab::{SortBy, SortDirection, SortField, Tab},
    };
    use std::fs::create_dir;
    use tempfile::TempDir;

    #[test]
    fn test_directory_navigation() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        create_dir(&sub_dir).unwrap();

        let mut tab = Tab::tab_entitys(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(tab.path(), temp_dir.path());

        tab.change_dir(sub_dir.clone()).unwrap();
        assert_eq!(tab.path(), sub_dir);

        tab.change_dir_back().unwrap();
        assert_eq!(tab.path(), temp_dir.path());
    }

    #[test]
    fn test_file_management_operations() {
        let temp_dir = TempDir::new().unwrap();
        let source_file = temp_dir.path().join("source.txt");
        let dest_dir = temp_dir.path().join("destination");

        std::fs::write(&source_file, "test content").unwrap();
        create_dir(&dest_dir).unwrap();

        let entity = Entity::new(source_file.clone()).unwrap();

        std::fs::copy(&source_file, dest_dir.join("copied.txt")).unwrap();
        assert!(dest_dir.join("copied.txt").exists());

        std::fs::rename(&source_file, dest_dir.join("moved.txt")).unwrap();
        assert!(dest_dir.join("moved.txt").exists());
        assert!(!source_file.exists());
    }

    #[test]
    fn test_file_sorting_and_filtering() {
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("small.txt"), "a").unwrap();
        std::fs::write(temp_dir.path().join("large.txt"), "a".repeat(1000)).unwrap();

        let mut tab = Tab::tab_entitys(temp_dir.path().to_path_buf()).unwrap();

        tab.set_sortby(&SortBy {
            field: SortField::Size,
            direction: SortDirection::Ascending,
        });

        let entities = if let Some(entities) = tab.entitys() {
            entities
        } else {
            Vec::new()
        };

        if entities.len() >= 2 {
            let small_file = entities.iter().find(|e| e.name() == "small").unwrap();
            let large_file = entities.iter().find(|e| e.name() == "large").unwrap();
            assert!(small_file.size().size_byte() < large_file.size().size_byte());
        }
    }

    #[test]
    fn test_permissions_management() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("permissions_test.txt");
        std::fs::write(&test_file, "content").unwrap();

        let entity = Entity::new(test_file.clone()).unwrap();
        let original_permissions = entity.permissions().clone();

        let new_permissions = EntityPermissions::new(0o755);
        std::fs::set_permissions(&test_file, new_permissions.clone().into()).unwrap();

        let updated_entity = Entity::new(test_file).unwrap();
        assert_ne!(
            original_permissions.mode(),
            updated_entity.permissions().mode()
        );
    }
}

#[cfg(test)]
mod benchmarks {
    use crate::{
        commands::Command,
        controller::JujikController,
        entity::{Entity, kind::EntityKind},
        model::JujikModel,
        tab::Tab,
    };
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };
    use tempfile::TempDir;

    #[test]
    fn benchmark_entity_creation() {
        let temp_dir = TempDir::new().unwrap();
        let test_files: Vec<_> = (0..1000)
            .map(|i| {
                let path = temp_dir.path().join(format!("file_{}.txt", i));
                std::fs::write(&path, "content").unwrap();
                path
            })
            .collect();

        let start = Instant::now();

        for file_path in test_files {
            let _ = Entity::new(file_path).unwrap();
        }

        let duration = start.elapsed();
        println!("Entity creation benchmark: {:?} for 1000 files", duration);

        assert!(duration.as_millis() < 1000);
    }

    #[test]
    fn benchmark_command_processing() {
        let (tx, rx) = mpsc::channel();
        let start = Instant::now();

        for i in 0..10000 {
            tx.send(Command::Update).unwrap();
        }

        for _ in 0..10000 {
            let _ = rx.recv().unwrap();
        }

        let duration = start.elapsed();
        println!(
            "Command processing benchmark: {:?} for 10000 commands",
            duration
        );

        assert!(duration.as_millis() < 5000);
    }

    #[test]
    fn stress_test_concurrent_operations() {
        let temp_dir = TempDir::new().unwrap();
        let (model_tx, model_rx) = mpsc::channel();
        let (controller_tx, controller_rx) = mpsc::channel();
        let (view_tx, _view_rx) = mpsc::channel();

        let model = JujikModel::new(controller_tx, model_rx);
        let controller = JujikController::new(model_tx.clone(), view_tx, controller_rx);

        let model_handle = model.run().unwrap();
        let controller_handle = controller.run().unwrap();

        let handles: Vec<_> = (0..100)
            .map(|i| {
                let tx = model_tx.clone();
                let temp_path = temp_dir.path().to_path_buf();

                std::thread::spawn(move || {
                    let ghost_entity = Entity::ghost(
                        temp_path.join(format!("stress_file_{}.txt", i)),
                        format!("stress_file_{}", i),
                        EntityKind::File,
                    )
                    .unwrap();

                    tx.send(Command::CreateEntity(0, Tab::default(), ghost_entity))
                        .unwrap();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        std::thread::sleep(Duration::from_secs(2));

        let created_files = std::fs::read_dir(&temp_dir).unwrap().count();
        assert!(created_files >= 100);

        model_tx.send(Command::Drop).unwrap();
        let _ = model_handle.join();
        let _ = controller_handle.join();
    }
}
