## JSON-Bucket

A simple app that connects to a MongoDB and acts as a "dropbox" for json blobs. This api accepts posts to the endpoint https://${server}/${collection}/create, and can query the posted documents using filters posted to http://${server}/collection/find or http://${server}/collection/findone. 

### Usage

json-bucket --db $MONGODB_DB --url $MONGODB_URI

--db: MongoDB database to utilize, can be passed as env var MONGODB_DB.
--uri: MongoDB uri, can be passed as env var MONGODB_URI

### ToDo

Future versions should have support for aggregations.
