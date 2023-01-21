from typing import Any


def missingKeys(dict:dict, keys:list[str])->list[str]:
	"""
	takes a dictionary and a list of keys

	returns a list of keys that were in the inputed list but not as keys in the dictionary
	"""
	return filter(lambda key:key in dict,keys)

def dictFill(dict:dict,key:str,value:Any)->None:
	"""
	takes a dict a key and a value
	
	if the dict is missing the key then it is asigned to the dictionary with the provided value
	"""
	if key not in dict: dict[key] = value
