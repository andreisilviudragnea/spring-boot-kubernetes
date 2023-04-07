./gradlew build
nerdctl --namespace k8s.io build --progress=plain -t demo:1.0 .
kubectl apply -f demo-deployment.yaml
kubectl get pods
