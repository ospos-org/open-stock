# Builds for development (Production builds should be done by CI)
docker build . -t bennjii/open-stock --build-arg RELEASE_TYPE=dev