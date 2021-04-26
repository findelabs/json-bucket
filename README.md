## JSON-Bucket

A simple app that connects to a MongoDB and acts as a "dropbox" for json blobs. This application exposes an ElasticSearch-inspired REST API, to enable quick and easy searches through http.


### Usage

Search for a single document in a collection, using MongoDB Regex:
```
curl -s localhost:8080/published/_find_one -d '{"summary": {"$regex": ".*interesting.*"}}'
```

Search for many documents in a collection, based on a simple query:
```
curl -s localhost:8080/published/_find -d '{"summary": "this is my summary"}'
```
Note: finds are limited to 100 returned docs for now.

Search for many documents, and and specify which fields to return:
```
curl -s localhost:8080/published/_find_project -d '[{"summary": {"$regex": ".*test.*"}},{"summary": 1, "_id": 0}]'
```

Return a count of documents in a collection:
```
curl -s localhost:8080/published/_count
```

### Running json-bucket
```
json-bucket --db $MONGODB_DB --url $MONGODB_URI
```
db: MongoDB database to utilize, can be passed as env var MONGODB_DB.
uri: MongoDB uri, can also be passed as env var MONGODB_URI

### ToDo

Future versions should have support for aggregations.
