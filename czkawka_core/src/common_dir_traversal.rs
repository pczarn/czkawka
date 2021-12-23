pub struct DirTraversalBuilder<F> {
    group_by: Option<F>,
    root_dirs: Vec<PathBuf>,
    stop_receiver: Option<&Receiver<()>>,
    progress_sender:  Option<&futures::channel::mpsc::UnboundedSender<ProgressData>>,
}

impl<F> DirTraversalBuilder<F> {
    fn new() -> Self {
        group_by: None,
        root_dirs: vec![],
        stop_receiver: None,
        progress_sender: None,
    }

    fn root_dirs(mut self, dirs: ) -> Self {
        self.root_dirs = Some(dirs);
        self
    }

    fn stop_receiver(mut self, stop_receiver: Option<&Receiver<()>>) -> Self {
        self.stop_receiver = Some(stop_receiver);
        self
    }

    fn progress_sender(mut self, progress_sender: Option<&futures::channel::mpsc::UnboundedSender<ProgressData>>) -> Self {
        self.progress_sender = Some(progress_sender);
        self
    }

    fn group_by<G>(self, group_by: G) -> DirTraversalBuilder<G>
        where G: Fn(&)
    {
        Self {
            group_by,
            root_dirs: self.root_dirs,
            stop_receiver: self.stop_receiver,
            progress_sender: self.progress_sender,
        }
    }
}


        let mut folders_to_check: Vec<PathBuf> = Vec::with_capacity(1024 * 2); // This should be small enough too not see to big difference and big enough to store most of paths without needing to resize vector

        // Add root folders for finding
        for id in &self.directories.included_directories {
            folders_to_check.push(id.clone());
        }

        //// PROGRESS THREAD START
        const LOOP_DURATION: u32 = 200; //in ms
        let progress_thread_run = Arc::new(AtomicBool::new(true));

        let atomic_file_counter = Arc::new(AtomicUsize::new(0));

        let progress_thread_handle = if let Some(progress_sender) = progress_sender {
            let progress_send = progress_sender.clone();
            let progress_thread_run = progress_thread_run.clone();
            let atomic_file_counter = atomic_file_counter.clone();
            thread::spawn(move || loop {
                progress_send
                    .unbounded_send(ProgressData {
                        checking_method: CheckingMethod::Name,
                        current_stage: 0,
                        max_stage: 0,
                        files_checked: atomic_file_counter.load(Ordering::Relaxed) as usize,
                        files_to_check: 0,
                    })
                    .unwrap();
                if !progress_thread_run.load(Ordering::Relaxed) {
                    break;
                }
                sleep(Duration::from_millis(LOOP_DURATION as u64));
            })
        } else {
            thread::spawn(|| {})
        };

        //// PROGRESS THREAD END

        while !folders_to_check.is_empty() {
            if stop_receiver.is_some() && stop_receiver.unwrap().try_recv().is_ok() {
                // End thread which send info to gui
                progress_thread_run.store(false, Ordering::Relaxed);
                progress_thread_handle.join().unwrap();
                return false;
            }

            let segments: Vec<_> = folders_to_check
                .par_iter()
                .map(|current_folder| {
                    let mut dir_result = vec![];
                    let mut warnings = vec![];
                    let mut fe_result = vec![];
                    // Read current dir childrens
                    let read_dir = match fs::read_dir(&current_folder) {
                        Ok(t) => t,
                        Err(e) => {
                            warnings.push(fl!(
                                "core_cannot_open_dir",
                                generate_translation_hashmap(vec![("dir", current_folder.display().to_string()), ("reason", e.to_string())])
                            ));
                            return (dir_result, warnings, fe_result);
                        }
                    };

                    // Check every sub folder/file/link etc.
                    'dir: for entry in read_dir {
                        let entry_data = match entry {
                            Ok(t) => t,
                            Err(e) => {
                                warnings.push(fl!(
                                    "core_cannot_read_entry_dir",
                                    generate_translation_hashmap(vec![("dir", current_folder.display().to_string()), ("reason", e.to_string())])
                                ));
                                continue 'dir;
                            }
                        };
                        let metadata: Metadata = match entry_data.metadata() {
                            Ok(t) => t,
                            Err(e) => {
                                warnings.push(fl!(
                                    "core_cannot_read_metadata_dir",
                                    generate_translation_hashmap(vec![("dir", current_folder.display().to_string()), ("reason", e.to_string())])
                                ));
                                continue 'dir;
                            }
                        };
                        if metadata.is_dir() {
                            if !self.recursive_search {
                                continue 'dir;
                            }

                            let next_folder = current_folder.join(entry_data.file_name());
                            if self.directories.is_excluded(&next_folder) {
                                continue 'dir;
                            }

                            if self.excluded_items.is_excluded(&next_folder) {
                                continue 'dir;
                            }

                            dir_result.push(next_folder);
                        } else if metadata.is_file() {
                            atomic_file_counter.fetch_add(1, Ordering::Relaxed);

                            let file_name_lowercase: String = match entry_data.file_name().into_string() {
                                Ok(t) => t,
                                Err(_inspected) => {
                                    warnings.push(fl!(
                                        "core_file_not_utf8_name",
                                        generate_translation_hashmap(vec![("name", entry_data.path().display().to_string())])
                                    ));
                                    continue 'dir;
                                }
                            }
                            .to_lowercase();

                            if !self.allowed_extensions.matches_filename(&file_name_lowercase) {
                                continue 'dir;
                            }

                            if (self.minimal_file_size..=self.maximal_file_size).contains(&metadata.len()) {
                                let current_file_name = current_folder.join(entry_data.file_name());
                                if self.excluded_items.is_excluded(&current_file_name) {
                                    continue 'dir;
                                }

                                let fe: FileEntry = FileEntry {
                                    path: current_file_name.clone(),
                                    size: metadata.len(),
                                    modified_date: match metadata.modified() {
                                        Ok(t) => match t.duration_since(UNIX_EPOCH) {
                                            Ok(d) => d.as_secs(),
                                            Err(_inspected) => {
                                                warnings.push(fl!(
                                                    "core_file_modified_before_epoch",
                                                    generate_translation_hashmap(vec![("name", current_file_name.display().to_string())])
                                                ));
                                                0
                                            }
                                        },
                                        Err(e) => {
                                            warnings.push(fl!(
                                                "core_file_no_modification_date",
                                                generate_translation_hashmap(vec![("name", current_file_name.display().to_string()), ("reason", e.to_string())])
                                            ));
                                            0
                                        }
                                    },
                                    hash: "".to_string(),
                                };

                                fe_result.push((entry_data.file_name().to_string_lossy().to_string(), fe));
                            }
                        }
                    }
                    (dir_result, warnings, fe_result)
                })
                .collect();

            // Advance the frontier
            folders_to_check.clear();

            // Process collected data
            for (segment, warnings, fe_result) in segments {
                folders_to_check.extend(segment);
                self.text_messages.warnings.extend(warnings);
                for (name, fe) in fe_result {
                    self.files_with_identical_names.entry(name.clone()).or_insert_with(Vec::new);
                    self.files_with_identical_names.get_mut(&name).unwrap().push(fe);
                }
            }
        }

        // End thread which send info to gui
        progress_thread_run.store(false, Ordering::Relaxed);
        progress_thread_handle.join().unwrap();