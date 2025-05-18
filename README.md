https://ryan-tock.github.io/binary-orbits/

zip lambda.zip lambda_function.py

docker build -t scipy-image .
docker run -d --name scipy-container scipy-image
docker cp scipy-container:/root/scipy-layer.zip .
docker stop scipy-container && docker rm scipy-container
tofu apply