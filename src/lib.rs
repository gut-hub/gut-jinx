use gut_lib;

gut_plugin::gut_export!(["jinx"], ["Shows the Jinx menu"]);

#[no_mangle]
fn jinx(ptr: *mut c_char) {
    let c_string = unsafe { CString::from_raw(ptr) };
    let _str_from_host = c_string.to_str().unwrap();

    let options = [
        "services list\t\t\tLists the services in jinx_conf.json".to_string(),
        "services rm\t\t\tRemoves a service from jinx_conf.json".to_string(),
        "services run\t\t\tRuns all services in jinx_conf.json".to_string(),
        "service add\t\t\tBuilds and adds service to jinx_conf.json".to_string(),
        "service build\t\t\tBuilds the service".to_string(),
        "init network\t\t\tCreates the docker swarm network".to_string(),
        "proxy build\t\t\tBuilds the jinx proxy".to_string(),
        "proxy run\t\t\tRuns the jinx proxy".to_string(),
        "cert generate\t\t\tGenerates certs for first time use".to_string(),
        "cert renew\t\t\tRenews certs (proxy running)".to_string(),
    ];

    let select = gut_lib::display::select_from_list(&options, None);
    println!("");

    match select {
        0 => services_list(),
        1 => services_rm(),
        2 => services_run(),
        3 => service_add(),
        4 => service_build(),
        5 => init_network(),
        6 => proxy_build(),
        7 => proxy_run(),
        8 => cert_generate(),
        9 => cert_renew(),
        _ => {}
    }
}

fn services_list() {
    // get jinx_conf
    let jinx_conf = jinx_proxy::conf::get_jinx_conf();

    // print out all the services
    for service in jinx_conf.jinx_services {
        println!("{:?}", service);
    }
}

fn services_rm() {
    // get jinx_conf
    let mut jinx_conf = jinx_proxy::conf::get_jinx_conf();

    // build a list of services
    let mut options = vec![];
    for service in &jinx_conf.jinx_services {
        options.push(format!("{:?}", service))
    }
    if options.is_empty() {
        return println!("No services in jinx_conf.json");
    }

    // prompt user for selection
    let select = gut_lib::display::select_from_list(&options, None);

    println!("Removing: {:?}", options[select]);

    // remove service and write to jinx_conf
    jinx_conf.jinx_services.remove(select);
    jinx_proxy::conf::write_jinx_conf(&jinx_conf);
}

pub fn services_run() {
    // get jinx_conf
    let jinx_conf = jinx_proxy::conf::get_jinx_conf();
    if jinx_conf.jinx_services.is_empty() {
        return println!("No services in jinx_conf.json");
    }

    // create services
    for service in &jinx_conf.jinx_services {
        println!("{:?}", &service);

        // get docker client
        let client = jinx_proxy::docker::get_client();

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime")
            .block_on(jinx_proxy::docker::create_service(client, &service));
    }
}

fn service_add() {
    // get current directory jinx.json
    let service = jinx_proxy::service::get_jinx_service();

    // get jinx_conf
    let mut jinx_conf = jinx_proxy::conf::get_jinx_conf();

    if jinx_conf.jinx_services.contains(&service) {
        return println!("Duplicate services in jinx_conf.json");
    }

    // build the service
    service_build();

    // write service to jinx_conf
    jinx_conf.jinx_services.push(service);
    jinx_proxy::conf::write_jinx_conf(&jinx_conf);
}

fn service_build() {
    // get current directory jinx.json
    let service = jinx_proxy::service::get_jinx_service();
    println!("{:?}", &service);

    // get dockerignore
    let dockerignore = jinx_proxy::docker::get_dockerignore();

    // create tar of directory
    println!("Creating project archive");
    jinx_proxy::targz::write_tar(&service, &dockerignore, None);

    // get tar bytes
    let tar = jinx_proxy::targz::get_tar(&service);

    // get docker client
    let client = jinx_proxy::docker::get_client();

    // build docker image
    println!("Building");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime")
        .block_on(jinx_proxy::docker::build_docker_image(
            client, &service, tar,
        ));
}

fn init_network() {
    // get docker client
    let client = jinx_proxy::docker::get_client();

    // create docker swarm network
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime")
        .block_on(jinx_proxy::docker::create_jinx_network(client));
}

fn proxy_build() {
    // get jinx_conf
    let jinx_conf = jinx_proxy::conf::get_jinx_conf();

    // create nginx conf from jinx_conf
    jinx_proxy::nginx::write_nginx_conf(&jinx_conf);

    // create jinx loadbalacer Dockerfile
    jinx_proxy::nginx::write_nginx_dockerfile();

    // create tar file
    println!("Creating proxy archive");
    jinx_proxy::targz::create_jinx_proxy_tar();

    // get tar bytes
    let tar = jinx_proxy::targz::get_jinx_proxy_tar();

    // get service
    let jinx_service = jinx_proxy::service::get_jinx_proxy_service();

    // get docker client
    let client = jinx_proxy::docker::get_client();

    // build docker image
    println!("Building");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime")
        .block_on(jinx_proxy::docker::build_docker_image(
            client,
            &jinx_service,
            tar,
        ));
}

pub fn proxy_run() {
    let service = jinx_proxy::service::get_jinx_proxy_service();
    println!("{:?}", &service);

    // get docker client
    let client = jinx_proxy::docker::get_client();

    // create docker service
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime")
        .block_on(jinx_proxy::docker::create_jinx_proxy_service(
            client, &service,
        ));
}

pub fn cert_generate() {
    // get jinx_files
    let jinx_files = jinx_proxy::file::get_jinx_files();

    // get jinx_conf
    let jinx_conf = jinx_proxy::conf::get_jinx_conf();

    for service in jinx_conf.jinx_services {
        // get docker client
        let client = jinx_proxy::docker::get_client();

        // certbot server ports
        let ports = vec!["80:80/tcp", "443:443/tcp"];

        // mount volumes
        let etc = format!("{}/letsencrypt:/etc/letsencrypt", jinx_files.jinx_home);
        let lib = format!("{}/letsencrypt:/var/lib/letsencrypt", jinx_files.jinx_home);
        let volumes = vec![etc.as_str(), lib.as_str()];

        let domain = format!("-d {}", service.domain);
        let www_domain = format!("-d www.{}", service.domain);

        let cmds = vec![
            "certonly",
            "--register-unsafely-without-email",
            "--agree-tos",
            "--standalone",
            domain.as_str(),
            www_domain.as_str(),
        ];

        // create docker service
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime")
            .block_on(jinx_proxy::docker::run_image(
                client,
                "certbot/certbot",
                ports,
                volumes,
                None,
                Some(cmds),
            ));
    }
}

pub fn cert_renew() {
    println!("Not implemented");
}
