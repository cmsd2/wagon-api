
set euxo pipefail

if [ "$CLIENT_ID" == "" ]; then
  echo "CLIENT_ID is not set"
  exit 1
fi

echo client id: $CLIENT_ID

read -p "Username: " username
read -s -p "Password: " password

aws cognito-idp initiate-auth \
--auth-flow USER_PASSWORD_AUTH \
--auth-parameters "USERNAME=$username,PASSWORD=$password" \
--client-id "$CLIENT_ID" \


