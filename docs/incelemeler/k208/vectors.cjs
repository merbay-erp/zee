const bcrypt = require('bcryptjs');
const cases = [['dogru-parola',11],['x'.repeat(72),4],['é'.repeat(36),4],['dogru-parola',14]];
const vectors = cases.map(([password,cost])=>({password,cost,hash:bcrypt.hashSync(password,cost)}));
process.stdout.write(JSON.stringify({producer:'bcryptjs 3.0.3 hashSync(password,cost)',node:process.version,vectors},null,2)+'\n');
