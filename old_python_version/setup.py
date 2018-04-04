#!/usr/bin/env python

from setuptools import setup, find_packages

setup(name='EvoFighters',
      version='0.5',
      description='RPG fighters who evolve to get better',
      author='Josh Kuhn',
      license='GPLv3',
      author_email='deontologician@gmail.com',
      packages=find_packages(),
      install_requires=[
          'blessings==1.5.1',
      ],
      entry_points={
          'console_scripts': [
              'EvoFighters = EvoFighters.Arena:main',
          ],
      },
      package_data = {
          '': ['*.ascii'],
      }
)
