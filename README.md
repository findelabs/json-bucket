## JSON-Bucket

A simple app that connects to a MongoDB and acts as a "dropbox" for json blobs. This application exposes an ElasticSearch-inspired REST API, to enable quick and easy searches through http.


### Interacting with Collections

Assuming that "published" is the name of a collection, then all of these apply:

**Create a new document in MongoDB**
```
curl localhost:8080/published/_insert -d '{"title": "This is a title", "summary": "The summary of the article"}'
```

**Search for a single document in a collection, using MongoDB Regex:**
```
curl -s localhost:8080/published/_find_one -d '{"summary": {"$regex": ".*article.*"}}'
```

**Search for many documents in a collection, based on a simple query:**
```
curl -s localhost:8080/published/_find -d '{"title": "This is a title"}'
```
Note: finds are limited to 100 returned docs for now.

**Search for many documents, and and specify which fields to return:**
```
curl -s localhost:8080/published/_find_project -d '[{"summary": {"$regex": ".*summary.*"}},{"summary": 1, "_id": 0}]'
```

**Search for one document, and and specify which fields to return:**
```
curl -s localhost:8080/published/_find_one_project -d '[{"summary": {"$regex": ".*summary.*"}},{"summary": 1, "_id": 0}]'
```

**Return a count of documents in a collection:**
```
curl -s localhost:8080/published/_count
```

**List indexes in collection:**
```
curl -s localhost:8080/published/_indexes
```

### Interacting with the Database

**List the collections in the database:**
```
curl -s localhost:8080/_cat/collections
```


### Running json-bucket
```
json-bucket --db $MONGODB_DB --url $MONGODB_URI
```
db: MongoDB database to utilize, can be passed as env var MONGODB_DB  
uri: MongoDB uri, can also be passed as env var MONGODB_URI
readonly: Access the database read-only

### ToDo

Future versions should have support for aggregations.
