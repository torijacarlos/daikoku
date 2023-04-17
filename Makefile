local-db: clean-db
	@./scripts/database.sh

clean-db: 
	@sudo docker stop daikoku || true;
	@sudo docker container prune -f 
