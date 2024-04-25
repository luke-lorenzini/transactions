import csv
import random

# Define the number of rows
num_rows = 100_000
types = ['deposit', 'withdrawal', 'dispute', 'resolve', 'chargeback']

# Generate a CSV file
with open('test_data_2.csv', 'w', newline='') as csvfile:
    writer = csv.writer(csvfile)
    
    # Write header
    writer.writerow(['type', 'client', 'tx', 'amount'])
    
    # Initialize starting TX number
    start_tx = 1000
    
    # Write rows
    for i in range(num_rows):
        # Generate random data for each column
        row = [
            random.choice(types),               # Type
            str(random.randint(0, 65535)),         # Client (using row index)
            str(start_tx + i),                  # TX (sequential)
            round(random.uniform(10, 100), 5)   # Amount (random float)
        ]
        writer.writerow(row)
