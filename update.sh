./gradlew build
nerdctl --namespace k8s.io build -t demo:1.4 .
kubectl apply -f demo-deployment.yaml
kubectl get pods
