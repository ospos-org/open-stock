
<div style="text-align:center">
   <img src="https://raw.githubusercontent.com/bennjii/open-stock/c811808ca63d75bfa99d5e4f032bab57dd997bec/docs/open-stock.svg#gh-dark-mode-only">
   <img src="https://raw.githubusercontent.com/bennjii/open-stock/c811808ca63d75bfa99d5e4f032bab57dd997bec/docs/open-stock-light.svg#gh-light-mode-only">
</div>

The open source inventory manager.

### Organise inventory, employees, customers and orders simply, reliably and fast. 

With `OpenStock`, you can manage:
- Stock Control
- Purchase Orders
- Point of Sale Operations
- Transactions
- Orders
- Employees (with Authentication)

In an API format. 
> For a visual interface, see [`open-pos`](https://github.com/bennjii/open-pos). 

OpenStock is a rust project utilizing [`rocket`](https://rocket.rs/) and [`sea-orm`](https://github.com/SeaQL/sea-orm) to batch and execute queries to a stored DB. 


> <mark>OpenStock is currently in Beta</mark>, so features may not yet be considered stable as many features are still considered experimental with the API formatting being largely subject to change. Thus, not ready for professional adoption as of yet.

## Getting Started
OpenStock is available as a crate to be integrated in your project or project-space if you wish to utilize the type-system created by it. Accessible by:

```
cargo add open-stock
```

However, for a default setup the API can be hosted yourself by performing the following:

1. Create an empty directory
2. In the directory, create a `docker-compose.yaml` (template file [here](./docker-compose.yaml)).
3. Customize it to add/remove `open-pos`, and `adminer`.
4. Include the default schema by creating a `/init` directory, and including [`01.sql`](./init/01.sql) inside.
5. Run `docker compose up`.

It is recommended to add a database viewer such as adminer for development to monitor changes and trace any issues of object structure you may encounter.

> Docker compose will come pre-installed with any install of [Docker Desktop](https://docs.docker.com/desktop/), but If you are on linux and do not have docker compose installed, you can install it from the guide [here](https://docs.docker.com/compose/install/#scenario-two-install-the-compose-plugin).

Please note:
1. `open-pos` is not required, but is a free POS system provided with the `open-stock` standard. You are free to use and/or modify it as necessary.

2. If you are launching the service from an ARM system, use `bennjii/open-stock:latest-arm` and `ghcr.io/bennjii/open-pos:latest-arm` for native performance.

3. If using `open-pos`, The `NEXT_PUBLIC_API_URL` environment URL refers to where the `open-stock` API is hosted. This is required for CORS, and is a required field. This can be a domain or IP.


## Setup Methods
<p align="center">
  <a href="#">
    
  </a>
  <p align="center">
   <img src="https://raw.githubusercontent.com/bennjii/open-stock/0c435b27ba3c17b46f3c142b60ccc036bc92d04d/docs/setup-method-banner.svg">
  </p>
</p>

It is important to consider how you wish to setup your provider. There are two methods of doing so shown below. A centralized approach will have a single server as a "source-of-truth", where each new retail location will interact with this one server. This is the most common approach and is the default implementation.

Alternatively, a fragmented approach can be undergone with replication layers existing at each store location or region to decrease server connection times due to large physical distances. 

> Please note, the fragmented approach has not been implemented yet but is planned for release in the future. 

## Migrating Information
If you need to migrate data from an existing provider, you can do so using the [migrator utility](https://github.com/bennjii/migrator). 

If you are using a provider not currently covered by the migration utility, you may request an implementation by publishing an issue with the tag, `feature-request` and title `+lang: <Existing Provider Name>`, and provide any associated information in the issue to help us implement the migration. Alternatively, you can implement the type conversions yourself and submit a pull request for it.

---

### Why rust?
Rust was the perfect choice for this project as it aims to produce a reliable and consistent output whilst taking various forms of input. Rust offers this in combination with high performance, albeit slower development times. However, for this project the trade-off is more than worth it. As rust has recently become a far more matured language, database ORM's like sea-orm (based on SQLx) and Diesel are well build and provide a high degree of type-safety in formatting, reading, writing and relaying information from the database - preventing poorly formatted entries and invalid column values. 