import json
import math
import secrets
from typing import List, Tuple
import threading

import helperFuncs as hf

class Game:
	__letters = ["a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z"]
	__numbers = ["0","1","2","3","4","5","6","7","8","9"]
	def __init__(self, setting:dict):
		self.settings:dict = setting
		self.addDefaults()
		self.pin:str = secrets.token_urlsafe(4)
		try:
			int(self.settings["size"])
			int(self.settings["players"])
		except ValueError:
			raise
		self.board = [["" for _ in range(int(self.settings["size"]))]for _ in range(int(self.settings["size"]))]
		self.players = ["" for _ in range(int(self.settings["players"]))]
	def addDefaults(self):
		hf.dictFill(self.settings,"size",8)
		hf.dictFill(self.settings,"disabled","")
		hf.dictFill(self.settings,"players",2)
	def getPeiceAtPos(pos):
		pass
	@staticmethod
	def toIndex(coord:str)->Tuple[int,int]:
		if type(coord)!=str:
			raise GameError("Error coord must be string")
		coord = coord.lower()
		xStr:str = ""
		i:int=0
		while (coord[i] in Game.__letters):
			xStr+=coord[i];
			i+=1
		if (xStr == ""):
			raise GameError("Error no letter component of the coord")
		yStr:str = coord[i]
		x:int = 0
		for i,xChar in enumerate(xStr):
			x+=Game.__letters.index(xChar) + len(Game.__letters)*i
		if (yStr == ""):
			raise GameError("Error no numeric component of the coord")
		for i,yChar in enumerate(yStr):
			if not(yChar in Game.__numbers):
				raise GameError("Error unknown character in coord")
		y:int = int(yStr)-1
		if (y < 0):
			raise GameError("Error invalid range for coord")
		return (x,y,)

	@staticmethod
	def toCoord(Index):
		if type(Index)!=tuple or len(Index)!=2 or type(Index[0])!=int or type(Index[1])!=int:
			raise GameError("Error `Index` must be a tuple of two ints")
		if (Index[0]<0 or Index[1]<0):
			raise GameError("Error: x and y must be larger than 0")
		return Game.__base(Index[0])+str(Index[1]+1)
	@staticmethod
	def __base(i):
		returnv:str=""
		if i>len(Game.__letters)-1:
			returnv = Game.__base(math.floor(i/len(Game.__letters))-1)
		i=i%len(Game.__letters)
		return returnv + Game.__letters[i]
	def ToJsonObject(self) -> str:
		return json.dumps(self.settings)

class GamesHander:
	def __init__(self):
		self.__Games = {}
		self.timeout = 30
	def add(self, g:Game):
		self.__Games[g.pin] = g
		threading.Timer(self.timeout,self.remove,[g.pin])
	def remove(self, pin):
		self.__Games[pin] = None
		del self.__Games[pin]
	def getKeys(self)->List:return list(self.__Games.keys())
	def getGame(self,key)->Game:return self.__Games[key]


class GameError(BaseException):pass