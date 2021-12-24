# 离线签名的 Claim

## 运行数据库

```bash
docker run \
    --restart always \
    --name claim_database \
    --volume /srv/claim:/var/lib/mysql \
    --publish 3306:3306 \
    --env MARIADB_ROOT_PASSWORD=biaoge \
    --env MARIADB_DATABASE=xprotocol \
    --detach \
    mariadb
```

## 表结构

```sql
CREATE TABLE IF NOT EXISTS `claims` (
  `id` bigint(50) unsigned NOT NULL AUTO_INCREMENT,
  `address` varchar(42) NOT NULL,
  `chain_id` varchar(50) NOT NULL,
  `token` varchar(42) NOT NULL,
  `nonce` bigint(50) unsigned NOT NULL DEFAULT 0,
  `number` bigint(50) unsigned NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`),
  UNIQUE KEY `atao` (`address`,`chain_id`,`token`,`nonce`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

## 获取网络列表

-   req

    ```http
    GET /chains
    ```

-   res

    ```json
    [
        {
            "chainId": "0x7b",
            "rpcUrls": ["https://testnet-eth.wlblock.io/"],
            "chainName": "Wlblock Test Chian",
            "default": true,
            "tokens": ["0xf7118ac23fa5e238e96d79a0504d7606effa2624"],
            "claim": "0x096bd91e851d3bd71a98053427da0394a732ab35"
        }
    ]
    ```

## 获取地址在改网络下可以提取多少

-   req

    ```http
    GET /{token}/{address}/{chain_id}/{nonce}
    ```

-   res

    就是一个数字

## 获取签名

-   req

    ```http
    GET /sign/{token}/{address}/{chain_id}/{nonce}
    ```

-   res

    ```json
    {
        "token": "0xf7118ac23fa5e238e96d79a0504d7606effa2624",
        "account": "0x76e552070892de9620d4bc68823e0c21232a8f60",
        "number": 1000000000,
        "nonce": 1,
        "v": 27,
        "r": "0xf3f10eb2301dccadc552d5c39c089c706c78006dbf7969a151bd7f68d0514b49",
        "s": "0x5f08d69e9921413c4f2179a91fb954a739f988cc3697acd21bf11a302b216116"
    }
    ```
