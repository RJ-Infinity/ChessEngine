from flask import Flask,render_template, request
import json
import functools
from game import Game,GamesHander
import helperFuncs as hf

Games = GamesHander()

server = Flask(__name__,static_folder="..\\web\\static", template_folder="..\\web\\template")

def POSTPath(func):
	@functools.wraps(func)
	def POSTPathInner(*args, **kwargs):
		if request.method != "POST":
			return "Error this URL should be accsessed via POST only"
		if request.data == "":
			return "{\"error\":\"no data recived\"}"
		try:
			data = json.loads(request.data)
		except json.decoder.JSONDecodeError:
			return "{\"error\":\"invalid json data\"}"
		return func(data, *args, **kwargs)
	return POSTPathInner

@server.route("/")
def homePage():
	return render_template('home.html')
@server.route("/new")
def newGamePage():
	return render_template("newGame.html")
@server.route("/join")
def joinGamePage():
	return render_template("joinGame.html")
@server.route("/settings")
def settingsPage():
	return render_template("settings.html")

@server.route('/createGame', methods=["POST"])
@POSTPath
def createGamePage(data):
	try:
		g = Game(data)
	except ValueError:
		return f"{{\"error\":\"values not in correct format\"}}"
	Games.add(g)
	return f"{{\"pin\":\"{g.pin}\"}}"
@server.route("/Game/<pin>")
def GamePage(pin):
	return render_template("Game.html",pin=pin)

@server.route("/joinGame", methods=["POST"])
@POSTPath
def joinGame(data):
	missingkeys = hf.missingKeys(data,["pin","userName"])
	if len(missingkeys) == 0:
		return f"{{\"error\":\"required keys not present in data {missingkeys}\"}}"
	if not data["pin"] in Games.getKeys():
		return f"{{\"error\":\"Game nonexistent\"}}"
	Games.getGame(data["pin"])
	return
@server.route("/getGameObject/<pin>")
def getGameObject(pin):
	if not pin in Games.getKeys():
		return f"{{\"error\":\"Game nonexistent\"}}"
	print("TEST")
	print(pin)
	print(Games.getGame(pin).ToJsonObject())
	return Games.getGame(pin).ToJsonObject()

if __name__ == "__main__":
	server.run(debug=True)