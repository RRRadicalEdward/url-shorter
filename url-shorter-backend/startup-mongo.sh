set -e
service mongod start
mongo <<EOF
set url-shorter
db.createUser({
  "user": "admin",
  "pwd":  "admin",
  roles: ["readWrite", "dbAdmin"]
});
db.createCollection("urls");
EOF
DATABASE_URL='mongodb://admin:admin@localhost:27017/url-shorter'
echo "DATABASE_URL - {$DATABASE_URL}"
