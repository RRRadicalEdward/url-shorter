set -e
#systemctl start mongod
mongo <<EOF
set url-shorter-tests
db.createUser({
  "user": "admin",
  "pwd":  "admin",
  roles: ["readWrite", "dbAdmin"]
});
db.test-urls.drop("test-urls")
db.createCollection("test-urls");
EOF
rm tests/test.env
touch tests/test.env
DATABASE_URL='mongodb://admin:admin@localhost:27017/url-shorter-tests'
echo "DATABASE_URL - $DATABASE_URL"
echo $DATABASE_URL >> tests/test.env
