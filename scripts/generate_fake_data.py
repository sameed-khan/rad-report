"""
generate_fake_data

Not critical to application performance. A simple script to load up the test database with test data for cases.
"""

import os
import random
import psycopg2 as psql
from dotenv import load_dotenv
from datetime import datetime, timedelta
from typing import Dict, Any
from tqdm import trange

def generate_random_row(start_date: datetime, 
                        end_date: datetime,
                        modality_probs: Dict[str, float],
                        subspecialty_probs: Dict[str, float],
                        npi: str = "1234567890",
                        exam_name: str = "FAKE_EXAM",
                        facility_name: str = "univ_hospitals") -> Dict[str, Any]:
                    
    random_date = start_date + timedelta(
        seconds=random.randint(0, int((end_date - start_date).total_seconds()))
    )
    modality = random.choices(population=list(modality_probs.keys()), weights=list(modality_probs.values()))
    subspecialty = random.choices(population=list(subspecialty_probs.keys()), weights=list(subspecialty_probs.values()))
    is_child = random.choices(population=[True, False], weights=[0.8, 0.2])

    return {
        "read_at": random_date,
        "npi": npi,
        "exam_name": exam_name,
        "modality": modality[0],
        "subspecialty": subspecialty[0],
        "is_child": is_child[0],
        "facility_name": facility_name
    }

def main():
    modprobs = {
        "MSK": 0.6,
        "Breast": 0.15,
        "Chest": 0.1,
        "Body": 0.05,
        "Neuro": 0.05,
        "Nuclear": 0.05,
    }
    ssprobs = {
        "XR": 0.6,
        "US": 0.15,
        "MRI": 0.1,
        "CT": 0.05,
        "Fluoro": 0.05,
        "SPECT": 0.05,
        "CT": 0.05,
    }

    stardate = datetime.strptime("2019-07-01", "%Y-%m-%d")
    endate = datetime.strptime("2024-06-30", "%Y-%m-%d")

    load_dotenv()
    database_url = os.getenv("DATABASE_URL")
    conn = psql.connect(database_url)
    cur = conn.cursor()

    for _ in trange(10721, desc="Inserting into database...", colour="green"):
        one_row = generate_random_row(stardate, endate, modprobs, ssprobs)
        cur.execute("INSERT INTO cases (read_at, npi, exam_name, modality, subspecialty, is_child, facility_name) VALUES (%s, %s, %s, %s, %s, %s, %s)",
                    (
                        one_row["read_at"], 
                        one_row["npi"], 
                        one_row["exam_name"], 
                        one_row["modality"], 
                        one_row["subspecialty"], 
                        one_row["is_child"], 
                        one_row["facility_name"]
                    )
        )

    conn.commit()
    cur.close()
    conn.close()

if __name__ == "__main__":
    main()