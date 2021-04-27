import math
apy = 1.1
dayi = apy ** (1 / 365)
monthi = dayi ** 30


def to_ratio(n):
    return f'Ratio::new_raw({math.floor(n*10**6)}, {10**6})'


print('const DAY_INTEREST: [Ratio<u128>; 30] = [')
for i in range(0, 30):
    di = dayi ** i
    print(f'{to_ratio(di)},')
print('];')

print('const MONTH_INTEREST: [Ratio<u128>; 12] = [')
for i in range(0, 12):
    mi = monthi ** i
    print(f'{to_ratio(mi)},')
print('];')

print('const YEAR_INTEREST: [Ratio<u128>; 100] = [')
for i in range(0, 100):
    yi = apy ** i
    print(f'{to_ratio(yi)},')
print('];')
