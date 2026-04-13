#!/bin/sh
 mkdir -p /etc/nginx/conf.d/ssl
 echo $DEFAULT_NGINX_KEY | base64 -d > /etc/nginx/conf.d/ssl/key.pem
 echo $DEFAULT_NGINX_CERT | base64 -d > /etc/nginx/conf.d/ssl/cert.pem