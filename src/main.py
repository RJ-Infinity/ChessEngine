from flask import Flask,render_template
import random

server = Flask(__name__,static_folder="..\\web\\static", template_folder="..\\web\\template")

@server.route("/")
def home():
	return render_template('home.html')
@server.route("/newGame")
def newGame():
	return render_template("newGame.html")
@server.route("/joinGame")
def joinGame():
	return render_template("joinGame.html")

if __name__ == "__main__":
	server.run(debug=True)