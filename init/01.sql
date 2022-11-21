CREATE TABLE IF NOT EXISTS `Products` (
  `sku` varchar(100) NOT NULL,
  `name` varchar(100) NOT NULL,
  `variants` varchar(255) NOT NULL,
  `loyalty_discount` text NOT NULL,
  `images` text NOT NULL,
  `tags` text NOT NULL,
  `description` text NOT NULL,
  `specifications` text NOT NULL,
  UNIQUE KEY `sku` (`sku`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Customer` (
  `id` varchar(100) NOT NULL,
  `name` text NOT NULL,
  `contact` text NOT NULL,
  `order_history` text NOT NULL,
  `customer_notes` text NOT NULL,
  `balance` int(11) NOT NULL,
  UNIQUE KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Employee` (
  `id` varchar(100) NOT NULL,
  `name` text NOT NULL,
  `contact` text NOT NULL,
  `clock_history` text NOT NULL,
  `level` int(11) NOT NULL,
  UNIQUE KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;

CREATE TABLE IF NOT EXISTS `Transactions` (
  `id` varchar(100) NOT NULL,
  `customer` text NOT NULL,
  `transaction_type` enum('in', 'out') NOT NULL,
  `products` text NOT NULL,
  `order_total` int(11) NOT NULL,
  `payment` text NOT NULL,
  `order_date` datetime NOT NULL,
  `order_notes` text NOT NULL,
  `order_history` text NOT NULL,
  `salesperson` text NOT NULL,
  `till` text NOT NULL,
  UNIQUE KEY `id` (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8;
