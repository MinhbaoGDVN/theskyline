cd "C:\Users\Admin\Desktop\Minhbao Projects\theskyline"
git add .
git commit -m "Cập nhật $(Get-Date -Format 'yyyy-MM-dd HH:mm')"
git pull --rebase origin main
git push