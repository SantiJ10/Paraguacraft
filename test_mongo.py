import pymongo
try:
    c = pymongo.MongoClient(
        "mongodb+srv://jalufsanti:Aminsj1001@cluster0.y5grb.mongodb.net/?appName=Cluster0",
        serverSelectionTimeoutMS=5000
    )
    info = c.server_info()
    print("Conexion exitosa. Version:", info.get("version"))
except Exception as e:
    print("Error:", e)
