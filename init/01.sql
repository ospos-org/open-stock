CREATE TABLE IF NOT EXISTS `Products` (
  `sku` varchar(100) NOT NULL,
  `name` varchar(100) NOT NULL,
  `company` text NOT NULL,
  `variants` json NOT NULL,
  `variant_groups` json NOT NULL,
  `images` json NOT NULL,
  `tags` json NOT NULL,
  `description` text NOT NULL,
  `specifications` json NOT NULL,
  PRIMARY KEY `sku` (`sku`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Customer` (
  `id` varchar(100) NOT NULL,
  `name` text NOT NULL,
  `contact` json NOT NULL,
  `customer_notes` json NOT NULL,
  `balance` FLOAT NOT NULL,
  `special_pricing` json NOT NULL,
  PRIMARY KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Transactions` (
  `id` varchar(100) NOT NULL,
  `customer` varchar(100) NOT NULL,
  `transaction_type` enum('in', 'out', 'pending-in', 'pending-out') NOT NULL,
  `products` json NOT NULL,
  `order_total` FLOAT NOT NULL,
  `payment` json NOT NULL,
  `order_date` datetime NOT NULL,
  `order_notes` json NOT NULL,
  `salesperson` text NOT NULL,
  `till` text NOT NULL,
  PRIMARY KEY `id` (`id`),
  FOREIGN KEY (customer) REFERENCES `Customer`(id)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Employee` (
  `id` varchar(100) NOT NULL,
  `name` json NOT NULL,
  `contact` json NOT NULL,
  `auth` json NOT NULL,
  `clock_history` json NOT NULL,
  `level` json NOT NULL,
  PRIMARY KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Supplier` (
  `id` varchar(100) NOT NULL,
  `name` json NOT NULL,
  `contact` json NOT NULL,
  `transaction_history` json NOT NULL,
  PRIMARY KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Session` (
  `id` varchar(100) NOT NULL PRIMARY KEY,
  -- The API key given
  `key` varchar(100) NOT NULL,
  -- The Employee's ID
  `employeeId` varchar(100) NOT NULL,
  -- When does the session expire
  `expiry` datetime NOT NULL,

  FOREIGN KEY (employeeId) REFERENCES Employee(id)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Store` (
  `id` varchar(100) NOT NULL,
  `name` text NOT NULL,
  `contact` json NOT NULL,
  `code` text NOT NULL,
  PRIMARY KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Promotion` (
  `id` varchar(100) NOT NULL,
  `name` text NOT NULL,
  `buy` json NOT NULL,
  `get` json NOT NULL,
  `valid_till` datetime NOT NULL,
  `timestamp` datetime NOT NULL,
  PRIMARY KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;