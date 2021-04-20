from ubuntu

RUN mkdir /app 

COPY entrypoint.sh /app/

RUN chmod +x /app/entrypoint.sh

COPY json-bucket /app/

ENTRYPOINT ["/app/entrypoint.sh"]

EXPOSE 8080
