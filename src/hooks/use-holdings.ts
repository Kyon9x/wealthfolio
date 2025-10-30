import { useQuery } from "@tanstack/react-query";
import { Holding } from "@/lib/types";
import { getHoldings } from "@/commands/portfolio";
import { QueryKeys } from "@/lib/query-keys";

export function useHoldings(accountId: string) {
  const {
    data: holdings = [],
    isLoading,
    isError,
    error,
  } = useQuery<Holding[], Error>({
    queryKey: [QueryKeys.HOLDINGS, accountId],
    queryFn: () => getHoldings(accountId),
    enabled: !!accountId,
  });

  // DEBUG POINT 1
  console.log('[useHoldings] Data fetched:', {
    count: holdings.length,
    accountId,
    firstHolding: holdings[0]
      ? {
          id: holdings[0].id,
          symbol: holdings[0].instrument?.symbol,
          holdingType: holdings[0].holdingType,
          marketValue: holdings[0].marketValue,
          rawMarketValue: JSON.stringify(holdings[0].marketValue),
        }
      : 'NO HOLDINGS',
  });

  return { holdings, isLoading, isError, error };
}
