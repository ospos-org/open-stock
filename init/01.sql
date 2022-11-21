CREATE TABLE IF NOT EXISTS `Products` (
  `sku` varchar(100) NOT NULL,
  `name` varchar(100) NOT NULL,
  `variants` json NOT NULL,
  `loyalty_discount` text NOT NULL,
  `images` json NOT NULL,
  `tags` json NOT NULL,
  `description` text NOT NULL,
  `specifications` json NOT NULL,
  UNIQUE KEY `sku` (`sku`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Customer` (
  `id` varchar(100) NOT NULL,
  `name` text NOT NULL,
  `contact` json NOT NULL,
  `order_history` json NOT NULL,
  `customer_notes` json NOT NULL,
  `balance` int(11) NOT NULL,
  UNIQUE KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Employee` (
  `id` varchar(100) NOT NULL,
  `name` text NOT NULL,
  `contact` json NOT NULL,
  `clock_history` json NOT NULL,
  `level` int(11) NOT NULL,
  UNIQUE KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Transactions` (
  `id` varchar(100) NOT NULL,
  `customer` text NOT NULL,
  `transaction_type` enum('in', 'out') NOT NULL,
  `products` json NOT NULL,
  `order_total` int(11) NOT NULL,
  `payment` json NOT NULL,
  `order_date` datetime NOT NULL,
  `order_notes` json NOT NULL,
  `order_history` json NOT NULL,
  `salesperson` text NOT NULL,
  `till` text NOT NULL,
  UNIQUE KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;
