# gut-jinx
Gut plugin for Jinx - Proxy and deploy docker containers

Jinx is a small proxy that runs on docker swarm. It will proxy traffic to the associated jinx services.
Jinx uses [letsencrypt](https://letsencrypt.org/) for certificate generation. Jinx configuration is located at: `$HOME/.jinx/jinx_conf.json`

### Use case
Jinx was created to have multiple services that listen on their own port be exposed to port 80,443 to the internet.
The services must only expose 1 port and the jinx proxy will foward incoming request for that domain to the service.

```
 ┌───────────┐
 │Jinx Proxy │ (port: 80,443)
 └─────┬─────┘
       │
       │
       │
       ▼
┌─────────────┐
│             ├─┐
│Jinx Services│ │ (port: 8080, 3000, etc)
│             │ │
└─┬───────────┘ │
  └─────────────┘
```

### Jinx Service
To create a Jinx Service, the file `jinx.json` must exist inside the project directory where the Dockerfile is located. It will compress the directory into a tar archive to be used with Docker's API to build the image.

see [jinx-service-example](https://github.com/gut-hub/jinx-service-example)

see [jinx.json](https://github.com/jareddlc/jinx/blob/master/src/service.rs#L10) definition


### Jinx Proxy
To create a Jinx Proxy, make sure to have added all the jinx services before building.
It will create the `nginx.conf` and `Dockerfile` necesary to build the jinx proxy. These files are stored in: `$HOME/.jinx/`


### Usage ([gut](https://github.com/gut-hub/gut)  needs to be installed):

Invoke jinx to perform one of the following commands `$ gut jinx`.

Commands:

```
services list   Lists the services in jinx_conf.json
services rm     Removes a service from jinx_conf.json
services run    Runs all services in jinx_conf.json
service add     Builds and adds service to jinx_conf.json
service build   Builds the service
init network    Creates the docker swarm network
proxy build     Builds the jinx proxy
proxy run       Runs the jinx proxy
cert generate   Generates certs for first time use
cert renew      Renews certs (proxy running)
```


#### Example workflow:

```shell
# initialize the jinx network
$ gut jinx (init network)

# contains jinx.json
$ cd my_project_dir

# add jinx service
$ gut jinx (service add)

# build jinx proxy
$ gut jinx (proxy build)

# run jinx services
$ gut jinx (services run)

# run jinx proxy
$ gut jinx (proxy run)
```

To generate certificates, make sure the DNS record is pointing to the host IP.
```shell
$ gut jinx (cert generate)
```

this will create a `letsencrypt` directory in the `$HOME/.jinx/` directory, and will contain the certificate files.
