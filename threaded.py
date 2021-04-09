import pip
import pandas as pd
import numpy as np
from pyinstrument import Profiler

# Read AIS data
df = pd.read_csv("data/ais.csv.gz")

# Multiply our test data by 1024x
for n in range(0, 10):
    df = pd.concat([df, df])

# Read in our five test polygons
with open ("data/tokyo.geojson", "r") as file: # 0
    tokyo = file.read()
with open ("data/channel.geojson", "r") as file: # 1
    channel = file.read()
with open ("data/hamburg.geojson", "r") as file: # 2
    hamburg = file.read()
with open ("data/athens.geojson", "r") as file: # 3
    athens = file.read()
with open ("data/singapore.geojson", "r") as file: # 4
    singapore = file.read()

# Start profiling
profiler = Profiler()
profiler.start()

# Pass an array of FeatureCollections in, one per polygon
engine = pip.Engine([tokyo, channel, hamburg, athens, singapore])

# Create numpy arrays of lat, lon coords
lat = df['lat'].values;
lon = df['lon'].values;

# Results contains a numpy array of polygon indexes, -1 meaning no polygon matched
df['polygon'] = engine.pip_n_threaded(lat, lon)

# Stop profiling
profiler.stop()

# Print profiler output
print(profiler.output_text(unicode=True, color=True))

# Print results, counts per polygon
print(df['polygon'].value_counts())

