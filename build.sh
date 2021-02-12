trap ERR

msg=$1

#if [[ $msg == "" ]]
#then 
#    echo "Please give commit message"
#    exit 1
#fi

cargo build --release

cp ./target/release/mongodb-poster .

strip mongodb-poster

sudo docker build .
