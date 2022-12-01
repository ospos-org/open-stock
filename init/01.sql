CREATE TABLE IF NOT EXISTS `Products` (
  `sku` varchar(100) NOT NULL,
  `name` varchar(100) NOT NULL,
  `company` text NOT NULL,
  `variants` json NOT NULL,
  `loyalty_discount` text NOT NULL,
  `images` json NOT NULL,
  `tags` json NOT NULL,
  `description` text NOT NULL,
  `specifications` json NOT NULL,
  PRIMARY KEY `sku` (`sku`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Customer` (
  `id` varchar(100) NOT NULL,
  `name` json NOT NULL,
  `contact` json NOT NULL,
  `order_history` json NOT NULL,
  `customer_notes` json NOT NULL,
  `balance` int(11) NOT NULL,
  `special_pricing` json NOT NULL,
  PRIMARY KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Employee` (
  `id` varchar(100) NOT NULL,
  `name` json NOT NULL,
  `contact` json NOT NULL,
  `auth` json NOT NULL,
  `clock_history` json NOT NULL,
  `level` int(11) NOT NULL,
  PRIMARY KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Supplier` (
  `id` varchar(100) NOT NULL,
  `name` json NOT NULL,
  `contact` json NOT NULL,
  `transaction_history` json NOT NULL,
  PRIMARY KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Transactions` (
  `id` varchar(100) NOT NULL,
  `customer` text NOT NULL,
  `transaction_type` enum('in', 'out', 'pending-in', 'pending-out') NOT NULL,
  `products` json NOT NULL,
  `order_total` int(11) NOT NULL,
  `payment` json NOT NULL,
  `order_date` datetime NOT NULL,
  `order_notes` json NOT NULL,
  `order_history` json NOT NULL,
  `salesperson` text NOT NULL,
  `till` text NOT NULL,
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

  -- impl! Add a timeout such that the API key NEEDS to be refreshed etc... + integrate handing API keys on requests.
) ENGINE = InnoDB DEFAULT CHARSET = utf8;