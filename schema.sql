#
# TABLE STRUCTURE FOR: Products
#

CREATE TABLE `Products` (
  `sku` bigint(20) NOT NULL,
  `name` varchar(100) NOT NULL,
  `variants` varchar(255) NOT NULL,
  `loyalty_discount` text NOT NULL,
  `images` text NOT NULL DEFAULT '[]',
  `tags` text NOT NULL DEFAULT '[]',
  `description` text NOT NULL,
  `specifications` text NOT NULL,
  UNIQUE KEY `sku` (`sku`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

#
# TABLE STRUCTURE FOR: customer
#

CREATE TABLE `customer` (
  `id` int(11) NOT NULL,
  `name` tinytext NOT NULL,
  `contact` text NOT NULL,
  `order_history` text NOT NULL DEFAULT '[]',
  `customer_notes` text NOT NULL,
  `balance` int(11) NOT NULL,
  UNIQUE KEY `id` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

#
# TABLE STRUCTURE FOR: employee
#

CREATE TABLE `employee` (
  `id` int(11) NOT NULL,
  `name` text NOT NULL,
  `contact` text NOT NULL DEFAULT '{}',
  `clock_history` text NOT NULL DEFAULT '[]',
  `level` int(11) NOT NULL,
  UNIQUE KEY `id` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

#
# TABLE STRUCTURE FOR: transactions
#

CREATE TABLE `transactions` (
  `id` int(9) unsigned NOT NULL AUTO_INCREMENT,
  `customer` text NOT NULL,
  `transaction_type` enum('in','out') NOT NULL,
  `products` text NOT NULL DEFAULT '{}',
  `order_total` int(11) NOT NULL,
  `payment` text NOT NULL DEFAULT '{}',
  `order_date` datetime NOT NULL,
  `order_notes` text NOT NULL,
  `salesperson` text NOT NULL,
  `till` text NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
